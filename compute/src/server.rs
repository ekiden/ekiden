use grpc;
use protobuf;
use protobuf::Message;
use std;

use std::fmt::Write;
use std::sync::Mutex;
use std::sync::mpsc::Sender;
use std::sync::mpsc::SyncSender;

use libcontract_common;
use libcontract_untrusted::enclave;

use generated::compute_web3::{CallContractRequest, CallContractResponse};
use generated::compute_web3_grpc::Compute;
use generated::consensus;
use generated::consensus_grpc;
use generated::consensus_grpc::Consensus;

/// This struct describes a call sent to the worker thread.
struct QueuedRequest {
    /// This is the request from the client.
    rpc_request: CallContractRequest,
    /// This is a channel where the worker should send the response.
    response_sender: SyncSender<grpc::SingleResponse<CallContractResponse>>,
}

/// This struct associates a response with a request.
struct QueuedResponse<'a> {
    /// This is the request. Notably, it owns the channel where we
    /// will be sending the response.
    queued_request: &'a QueuedRequest,
    /// This is the response.
    grpc_response: grpc::SingleResponse<CallContractResponse>,
}

struct CachedStateInitialized {
    encrypted_state: libcontract_common::api::CryptoSecretbox,
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
    ins: super::instrumentation::WorkerMetrics,
}

impl ComputeServerWorker {
    fn new(contract_filename: String, consensus_host: String, consensus_port: u16) -> Self {
        // Connect to consensus node
        ComputeServerWorker {
            contract: Self::create_contract(&contract_filename),
            cached_state: None,
            ins: super::instrumentation::WorkerMetrics::new(),
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

    fn call_contract_fallible(
        &self,
        encrypted_state_opt: Option<libcontract_common::api::CryptoSecretbox>,
        rpc_request: &CallContractRequest,
    ) -> Result<
        (
            Option<libcontract_common::api::CryptoSecretbox>,
            CallContractResponse,
        ),
        Box<std::error::Error>,
    > {
        let mut enclave_request = libcontract_common::api::EnclaveRequest::new();
        let client_request = protobuf::parse_from_bytes(rpc_request.get_payload())?;
        enclave_request.set_client_request(client_request);
        if let Some(encrypted_state) = encrypted_state_opt {
            enclave_request.set_encrypted_state(encrypted_state);
        }

        let enclave_request_bytes = enclave_request.write_to_bytes()?;
        let enclave_response_bytes = {
            let _enclave_timer = self.ins.req_time_enclave.start_timer();
            self.contract.call_raw(enclave_request_bytes)
        }?;

        let mut enclave_response: libcontract_common::api::EnclaveResponse =
            protobuf::parse_from_bytes(&enclave_response_bytes)?;
        let mut rpc_response = CallContractResponse::new();
        rpc_response.set_payload(enclave_response.get_client_response().write_to_bytes()?);
        let new_encrypted_state_opt = if enclave_response.has_encrypted_state() {
            Some(enclave_response.take_encrypted_state())
        } else {
            None
        };

        Ok((new_encrypted_state_opt, rpc_response))
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
    ) -> Result<(), Box<std::error::Error>> {
        self.cached_state = Some(CachedStateInitialized {
            encrypted_state: protobuf::parse_from_bytes(checkpoint.get_payload())?,
            height: checkpoint.get_height(),
        });
        Ok(())
    }

    fn advance_cached_state(
        &mut self,
        diffs: &[Vec<u8>],
    ) -> Result<libcontract_common::api::CryptoSecretbox, Box<std::error::Error>> {
        #[cfg(feature = "no_diffs")]
        assert!(
            diffs.is_empty(),
            "attempted to apply diffs in a no_diffs build"
        );

        let csi = self.cached_state
            .as_mut()
            .ok_or::<Box<std::error::Error>>(From::from(
                "advance_cached_state called with uninitialized cached state",
            ))?;
        for diff in diffs {
            let mut res: libcontract_common::api::StateApplyResponse =
                self.contract
                    .call(libcontract_common::api::METHOD_STATE_APPLY, &{
                        let mut req = libcontract_common::api::StateApplyRequest::new();
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
        request_batch: &'a [QueuedRequest],
    ) -> Result<Vec<QueuedResponse<'a>>, Box<std::error::Error>> {
        // Get state updates from consensus
        let mut encrypted_state_opt = if self.consensus.is_some() {
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

        // Process the requests.
        let mut ever_update_state = false;
        let response_batch = request_batch
            .iter()
            .map(|ref queued_request| {
                let grpc_response = match self.call_contract_fallible(
                    encrypted_state_opt.clone(),
                    &queued_request.rpc_request,
                ) {
                    Ok((new_encrypted_state_opt, rpc_response)) => {
                        if let Some(new_encrypted_state) = new_encrypted_state_opt {
                            encrypted_state_opt = Some(new_encrypted_state);
                            ever_update_state = true;
                        }
                        grpc::SingleResponse::completed(rpc_response)
                    }
                    Err(e) => {
                        eprintln!("compute: error in call {:?}", e);
                        grpc::SingleResponse::err(grpc::Error::Panic(String::from(e.description())))
                    }
                };
                QueuedResponse {
                    queued_request,
                    grpc_response,
                }
            })
            .collect();

        // Set state in consensus
        if let Some(encrypted_state) = encrypted_state_opt {
            if ever_update_state {
                let _consensus_set_timer = self.ins.consensus_set_time.start_timer();
                match orig_encrypted_state_opt {
                    Some(orig_encrypted_state) => {
                        let diff_res: libcontract_common::api::StateDiffResponse =
                            self.contract.call(libcontract_common::api::METHOD_STATE_DIFF, &{
                                let mut diff_req = libcontract_common::api::StateDiffRequest::new();
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
        }

        Ok(response_batch)
    }

    fn call_contract_batch(&mut self, request_batch: Vec<QueuedRequest>) {
        match self.call_contract_batch_fallible(&request_batch) {
            Ok(response_batch) => {
                // No batch-wide errors. Send out per-call responses.
                for queued_response in response_batch {
                    queued_response
                        .queued_request
                        .response_sender
                        .send(queued_response.grpc_response)
                        .unwrap();
                }
            }
            Err(e) => {
                eprintln!("compute: batch-wide error {:?}", e);
                // Send batch-wide error to all clients.
                let desc = String::from(e.description());
                for queued_request in &request_batch {
                    let grpc_response = grpc::SingleResponse::err(grpc::Error::Panic(desc.clone()));
                    queued_request.response_sender.send(grpc_response).unwrap();
                }
            }
        }
    }

    /// Process requests from a receiver until the channel closes.
    fn work(&mut self, request_receiver: std::sync::mpsc::Receiver<QueuedRequest>) {
        // Block for the next call.
        while let Ok(queued_request) = request_receiver.recv() {
            self.ins.reqs_batches_started.inc();
            let _batch_timer = self.ins.req_time_batch.start_timer();
            let mut request_batch = Vec::new();
            request_batch.push(queued_request);
            // Additionally dequeue any remaining requests.
            while let Ok(queued_request) = request_receiver.try_recv() {
                request_batch.push(queued_request);
            }
            // Process the requests.
            self.call_contract_batch(request_batch);
        }
    }
}

pub struct ComputeServerImpl {
    /// Channel for submitting requests to the worker.
    request_sender: Mutex<Sender<QueuedRequest>>,
    /// Instrumentation objects.
    ins: super::instrumentation::HandlerMetrics,
}

impl ComputeServerImpl {
    /// Create new compute server instance.
    pub fn new(contract_filename: &str, consensus_host: &str, consensus_port: u16) -> Self {
        let contract_filename_owned = String::from(contract_filename);
        let consensus_host_owned = String::from(consensus_host);
        let (request_sender, request_receiver) = std::sync::mpsc::channel();
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
            ins: super::instrumentation::HandlerMetrics::new(),
        }
    }
}

impl Compute for ComputeServerImpl {
    fn call_contract(
        &self,
        _options: grpc::RequestOptions,
        rpc_request: CallContractRequest,
    ) -> grpc::SingleResponse<CallContractResponse> {
        self.ins.reqs_received.inc();
        let _client_timer = self.ins.req_time_client.start_timer();
        let (response_sender, response_receiver) = std::sync::mpsc::sync_channel(0);
        {
            let request_sender = self.request_sender.lock().unwrap();
            request_sender
                .send(QueuedRequest {
                    rpc_request,
                    response_sender,
                })
                .unwrap();
        }
        response_receiver.recv().unwrap()
    }
}
