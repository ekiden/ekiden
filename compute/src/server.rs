use grpc;

use protobuf;
use protobuf::Message;

use thread_local::ThreadLocal;

use futures::Future;
use futures::sync::oneshot;

use time;

use std;
use std::error::Error;
use std::fmt::Write;
use std::sync::Mutex;
use std::sync::mpsc::{channel, Receiver, Sender};

use libcontract_common::api;
use libcontract_untrusted::enclave;

use super::generated::compute_web3::{CallContractRequest, CallContractResponse};
use super::generated::compute_web3_grpc::Compute;
use super::generated::consensus;
use super::generated::consensus_grpc;
use super::generated::consensus_grpc::Consensus;
use super::instrumentation;

/// This struct describes a call sent to the worker thread.
struct QueuedRequest {
    /// This is the request from the client.
    rpc_request: CallContractRequest,
    /// This is a channel where the worker should send the response. The channel is only
    /// available until it has been used for sending a response and is None afterwards.
    response_sender:
        Option<oneshot::Sender<Result<CallContractResponse, Box<Error + Sync + Send + 'static>>>>,
}

/// This struct associates a response with a request.
struct QueuedResponse<'a> {
    /// This is the request. Notably, it owns the channel where we
    /// will be sending the response.
    queued_request: &'a mut QueuedRequest,
    /// This is the response.
    response: Result<CallContractResponse, Box<Error + Sync + Send + 'static>>,
}

struct CachedStateInitialized {
    encrypted_state: api::CryptoSecretbox,
    height: u64,
}

struct ComputeServerWorker {
    /// Consensus client.
    consensus: Option<consensus_grpc::ConsensusClient>,
    /// Contract running in an enclave.
    contract: enclave::EkidenEnclave,
    /// Cached state reconstituted from checkpoint and diffs. None if
    /// cache or state is uninitialized.
    cached_state: Option<CachedStateInitialized>,
    /// Instrumentation objects.
    ins: instrumentation::WorkerMetrics,
}

impl ComputeServerWorker {
    // TODO: Make these runtime configurable.
    /// Maximum batch size (number of requests).
    const MAX_BATCH_SIZE: usize = 1000;
    /// Maximum batch timeout (in nsec).
    const MAX_BATCH_TIMEOUT: u64 = 1000 * 1_000_000;

    fn new(contract_filename: String, consensus_host: String, consensus_port: u16) -> Self {
        // Connect to consensus node
        ComputeServerWorker {
            contract: Self::create_contract(&contract_filename),
            cached_state: None,
            ins: instrumentation::WorkerMetrics::new(),
            // TODO: Use TLS client.
            consensus: match consensus_grpc::ConsensusClient::new_plain(
                &consensus_host,
                consensus_port,
                Default::default(),
            ) {
                Ok(client) => Some(client),
                _ => {
                    eprintln!(
                        "WARNING: Failed to create consensus client. No state will be fetched."
                    );

                    None
                }
            },
        }
    }

    /// Create an instance of the contract.
    fn create_contract(contract_filename: &str) -> enclave::EkidenEnclave {
        // TODO: Handle contract initialization errors.
        let contract = enclave::EkidenEnclave::new(contract_filename).unwrap();

        // Initialize contract.
        // TODO: Support contract restore.
        let response = contract
            .initialize()
            .expect("Failed to initialize contract");

        // Show contract MRENCLAVE in hex format.
        let mut mr_enclave = String::new();
        for &byte in response.get_mr_enclave() {
            write!(&mut mr_enclave, "{:02x}", byte).unwrap();
        }

        println!("Loaded contract with MRENCLAVE: {}", mr_enclave);

        contract
    }

    #[cfg(not(feature = "no_cache"))]
    fn get_cached_state_height(&self) -> Option<u64> {
        match self.cached_state.as_ref() {
            Some(csi) => Some(csi.height),
            None => None,
        }
    }

    fn set_cached_state(
        &mut self,
        checkpoint: &consensus::Checkpoint,
    ) -> Result<(), Box<Error + Sync + Send + 'static>> {
        self.cached_state = Some(CachedStateInitialized {
            encrypted_state: protobuf::parse_from_bytes(checkpoint.get_payload())?,
            height: checkpoint.get_height(),
        });
        Ok(())
    }

    fn advance_cached_state(
        &mut self,
        diffs: &[Vec<u8>],
    ) -> Result<api::CryptoSecretbox, Box<Error + Sync + Send + 'static>> {
        #[cfg(feature = "no_diffs")]
        assert!(
            diffs.is_empty(),
            "attempted to apply diffs in a no_diffs build"
        );

        let csi = self.cached_state
            .as_mut()
            .ok_or::<Box<Error + Sync + Send + 'static>>(From::from(
                "advance_cached_state called with uninitialized cached state",
            ))?;
        for diff in diffs {
            let mut res: api::StateApplyResponse = self.contract.call(api::METHOD_STATE_APPLY, &{
                let mut req = api::StateApplyRequest::new();
                req.set_old(csi.encrypted_state.clone());
                req.set_diff(protobuf::parse_from_bytes(diff)?);
                req
            })?;
            csi.encrypted_state = res.take_new();
            csi.height += 1;
        }
        Ok(csi.encrypted_state.clone())
    }

    fn call_contract_batch_fallible<'a>(
        &mut self,
        request_batch: &'a mut [QueuedRequest],
    ) -> Result<Vec<QueuedResponse<'a>>, Box<Error + Sync + Send + 'static>> {
        // Get state updates from consensus
        let encrypted_state_opt = if self.consensus.is_some() {
            let _consensus_get_timer = self.ins.consensus_get_time.start_timer();

            #[cfg(not(feature = "no_cache"))]
            let cached_state_height = self.get_cached_state_height();
            #[cfg(feature = "no_cache")]
            let cached_state_height = None;

            match cached_state_height {
                Some(height) => {
                    let (_, consensus_response, _) = self.consensus
                        .as_ref()
                        .unwrap()
                        .get_diffs(grpc::RequestOptions::new(), {
                            let mut consensus_request = consensus::GetDiffsRequest::new();
                            consensus_request.set_since_height(height);
                            consensus_request
                        })
                        .wait()?;
                    if consensus_response.has_checkpoint() {
                        self.set_cached_state(consensus_response.get_checkpoint())?;
                    }
                    Some(self.advance_cached_state(consensus_response.get_diffs())?)
                }
                None => {
                    if let Ok((_, consensus_response, _)) = self.consensus
                        .as_ref()
                        .unwrap()
                        .get(grpc::RequestOptions::new(), consensus::GetRequest::new())
                        .wait()
                    {
                        self.set_cached_state(consensus_response.get_checkpoint())?;
                        Some(self.advance_cached_state(consensus_response.get_diffs())?)
                    } else {
                        // We should bail if there was an error other
                        // than the state not being initialized. But
                        // don't go fixing this. There's another
                        // resolution planned in #95.
                        None
                    }
                }
            }
        } else {
            None
        };

        #[cfg(not(feature = "no_diffs"))]
        let orig_encrypted_state_opt = encrypted_state_opt.clone();
        #[cfg(feature = "no_diffs")]
        let orig_encrypted_state_opt = None;

        // Call contract with batch of requests.
        let mut enclave_request = api::EnclaveRequest::new();

        // Prepare batch of requests.
        {
            let client_requests = enclave_request.mut_client_request();
            for ref queued_request in request_batch.iter() {
                // TODO: Why doesn't enclave request contain bytes directly?
                let client_request =
                    protobuf::parse_from_bytes(queued_request.rpc_request.get_payload())?;
                client_requests.push(client_request);
            }
        }

        // Add state if it is available.
        if let Some(encrypted_state) = encrypted_state_opt {
            enclave_request.set_encrypted_state(encrypted_state);
        }

        let enclave_request_bytes = enclave_request.write_to_bytes()?;
        let enclave_response_bytes = {
            let _enclave_timer = self.ins.req_time_enclave.start_timer();
            self.contract.call_raw(enclave_request_bytes)
        }?;

        let mut enclave_response: api::EnclaveResponse =
            protobuf::parse_from_bytes(&enclave_response_bytes)?;

        // Assert equal number of responses, fail otherwise (corrupted response).
        if enclave_response.get_client_response().len() != request_batch.len() {
            // TODO: Use proper error class.
            return Err(Box::new(grpc::Error::Panic(
                "Corrupted response (response count != request count)".to_string(),
            )));
        }

        let mut response_batch = vec![];
        for (index, queued_request) in request_batch.iter_mut().enumerate() {
            let mut response = CallContractResponse::new();
            // TODO: Why doesn't enclave response contain bytes directly?
            response
                .set_payload((&enclave_response.get_client_response()[index]).write_to_bytes()?);

            response_batch.push(QueuedResponse {
                queued_request,
                response: Ok(response),
            });
        }

        // Check if any state was produced. In case no state was produced, this means that
        // no request caused a state update and thus no state update is required.
        if enclave_response.has_encrypted_state() {
            let encrypted_state = enclave_response.take_encrypted_state();

            let _consensus_set_timer = self.ins.consensus_set_time.start_timer();
            match orig_encrypted_state_opt {
                Some(orig_encrypted_state) => {
                    let diff_res: api::StateDiffResponse =
                        self.contract.call(api::METHOD_STATE_DIFF, &{
                            let mut diff_req = api::StateDiffRequest::new();
                            diff_req.set_old(orig_encrypted_state);
                            diff_req.set_new(encrypted_state);
                            diff_req
                        })?;
                    self.consensus
                        .as_ref()
                        .unwrap()
                        .add_diff(grpc::RequestOptions::new(), {
                            let mut add_diff_req = consensus::AddDiffRequest::new();
                            add_diff_req.set_payload(diff_res.get_diff().write_to_bytes()?);
                            add_diff_req
                        })
                        .wait()?;
                }
                None => {
                    let mut consensus_replace_request = consensus::ReplaceRequest::new();
                    consensus_replace_request.set_payload(encrypted_state.write_to_bytes()?);
                    self.consensus
                        .as_ref()
                        .unwrap()
                        .replace(grpc::RequestOptions::new(), consensus_replace_request)
                        .wait()?;
                }
            }
        }

        Ok(response_batch)
    }

    fn call_contract_batch(&mut self, mut request_batch: Vec<QueuedRequest>) {
        // Contains a batch-wide error if one has occurred.
        let batch_error: Option<String>;

        {
            match self.call_contract_batch_fallible(&mut request_batch) {
                Ok(response_batch) => {
                    // No batch-wide errors. Send out per-call responses.
                    for queued_response in response_batch {
                        let sender = queued_response
                            .queued_request
                            .response_sender
                            .take()
                            .unwrap();
                        sender.send(queued_response.response).unwrap();
                    }

                    return;
                }
                Err(error) => {
                    // Batch-wide error has occurred. We cannot handle the error here as we
                    // must first drop the mutable request_batch reference.
                    eprintln!("compute: batch-wide error {:?}", error);

                    batch_error = Some(String::from(error.description()));
                }
            }
        }

        // Send batch-wide error to all clients.
        let batch_error = batch_error.as_ref().unwrap();
        for mut queued_request in request_batch {
            let sender = queued_request.response_sender.take().unwrap();
            sender
                // TODO: Use proper error class.
                .send(Err(Box::new(grpc::Error::Panic(batch_error.clone()))))
                .unwrap();
        }
    }

    /// Process requests from a receiver until the channel closes.
    fn work(&mut self, request_receiver: Receiver<QueuedRequest>) {
        // Block for the next call.
        while let Ok(queued_request) = request_receiver.recv() {
            self.ins.reqs_batches_started.inc();
            let _batch_timer = self.ins.req_time_batch.start_timer();

            let mut request_batch = Vec::new();
            request_batch.push(queued_request);

            // Queue up requests up to MAX_BATCH_SIZE, but for at most MAX_BATCH_TIMEOUT.
            let batch_start = time::precise_time_ns();
            while request_batch.len() < Self::MAX_BATCH_SIZE
                && time::precise_time_ns() - batch_start < Self::MAX_BATCH_TIMEOUT
            {
                while request_batch.len() < Self::MAX_BATCH_SIZE {
                    if let Ok(queued_request) = request_receiver.try_recv() {
                        request_batch.push(queued_request);
                    } else {
                        break;
                    }
                }

                // Yield thread for 10 ms while we wait.
                std::thread::sleep(std::time::Duration::from_millis(10));
            }

            // Process the requests.
            self.call_contract_batch(request_batch);
        }
    }
}

pub struct ComputeServerImpl {
    /// Channel for submitting requests to the worker. This is only used to
    /// initialize a thread-local clone of the sender handle, so that there
    /// is no need for locking during request processing.
    request_sender: Mutex<Sender<QueuedRequest>>,
    /// Thread-local channel for submitting requests to the worker.
    tl_request_sender: ThreadLocal<Sender<QueuedRequest>>,
    /// Instrumentation objects.
    ins: instrumentation::HandlerMetrics,
}

impl ComputeServerImpl {
    /// Create new compute server instance.
    pub fn new(contract_filename: &str, consensus_host: &str, consensus_port: u16) -> Self {
        let contract_filename_owned = String::from(contract_filename);
        let consensus_host_owned = String::from(consensus_host);

        let (request_sender, request_receiver) = channel();
        // move request_receiver
        std::thread::spawn(move || {
            ComputeServerWorker::new(
                contract_filename_owned,
                consensus_host_owned,
                consensus_port,
            ).work(request_receiver);
        });

        ComputeServerImpl {
            request_sender: Mutex::new(request_sender),
            tl_request_sender: ThreadLocal::new(),
            ins: instrumentation::HandlerMetrics::new(),
        }
    }

    /// Get thread-local request sender.
    fn get_request_sender(&self) -> &Sender<QueuedRequest> {
        self.tl_request_sender.get_or(|| {
            // Only take the lock when we need to clone the sender for a new thread.
            let request_sender = self.request_sender.lock().unwrap();
            Box::new(request_sender.clone())
        })
    }
}

impl Compute for ComputeServerImpl {
    fn call_contract(
        &self,
        _options: grpc::RequestOptions,
        rpc_request: CallContractRequest,
    ) -> grpc::SingleResponse<CallContractResponse> {
        // Instrumentation.
        self.ins.reqs_received.inc();
        let _client_timer = self.ins.req_time_client.start_timer();

        // Send request to worker thread.
        let (response_sender, response_receiver) = oneshot::channel();
        self.get_request_sender()
            .send(QueuedRequest {
                rpc_request,
                response_sender: Some(response_sender),
            })
            .unwrap();

        // Prepare response future.
        grpc::SingleResponse::no_metadata(response_receiver.then(|result| match result {
            Ok(Ok(response)) => Ok(response),
            Ok(Err(error)) => Err(grpc::Error::Panic(error.description().to_string())),
            Err(error) => Err(grpc::Error::Panic(error.description().to_string())),
        }))
    }
}
