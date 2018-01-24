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

use generated::compute_web3::{CallContractRequest, CallContractResponse, IasGetSpidRequest,
                              IasGetSpidResponse, IasVerifyQuoteRequest, IasVerifyQuoteResponse,
                              IasVerifyQuoteResponse_Status};
use generated::compute_web3_grpc::Compute;
use generated::consensus;
use generated::consensus_grpc;
use generated::consensus_grpc::Consensus;

use super::ias::{IASConfiguration, IAS};

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

pub struct ComputeServerImpl {
    // Channel for submitting requests to the worker.
    request_sender: Mutex<Sender<QueuedRequest>>,
    // IAS service.
    ias: IAS,
}

impl ComputeServerImpl {
    /// Create new compute server instance.
    pub fn new(contract_filename: &str, ias: IASConfiguration) -> Self {
        let contract_filename_owned = String::from(contract_filename);
        let (request_sender, request_receiver) = std::sync::mpsc::channel();
        // move request_receiver
        std::thread::spawn(move || {
            let contract = Self::get_contract(&contract_filename_owned);
            // Block for the next call.
            // When ComputeServerImpl is dropped, the request_sender closes, and the thread will exit.
            while let Ok(queued_request) = request_receiver.recv() {
                let mut request_batch = Vec::new();
                request_batch.push(queued_request);
                // Additionally dequeue any remaining requests.
                while let Ok(queued_request) = request_receiver.try_recv() {
                    request_batch.push(queued_request);
                }
                // Process the requests.
                Self::call_contract_batch(&contract, request_batch);
            }
        });
        ComputeServerImpl {
            request_sender: Mutex::new(request_sender),
            ias: IAS::new(ias).unwrap(),
        }
    }

    /// Get thread-local instance of the contract.
    fn get_contract(contract_filename: &str) -> enclave::EkidenEnclave {
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
        contract: &enclave::EkidenEnclave,
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
        let enclave_response_bytes = contract.call_raw(&enclave_request_bytes)?;

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

    fn call_contract_batch_fallible<'a>(
        contract: &enclave::EkidenEnclave,
        request_batch: &'a [QueuedRequest],
    ) -> Result<Vec<QueuedResponse<'a>>, Box<std::error::Error>> {
        // Connect to consensus node
        // TODO: Know the consensus node location other than having it hard-coded.
        // TODO: Use TLS client.
        let consensus_client =
            consensus_grpc::ConsensusClient::new_plain("localhost", 9002, Default::default())?;

        // Get state from consensus
        let consensus_result = consensus_client
            .get(grpc::RequestOptions::new(), consensus::GetRequest::new())
            .wait();
        let mut encrypted_state_opt = if let Ok((_, consensus_get_response, _)) = consensus_result {
            let encrypted_state = protobuf::parse_from_bytes(consensus_get_response.get_payload())?;
            Some(encrypted_state)
        } else {
            // We should bail if there was an error other than the state not being initialized.
            // But don't go fixing this. There's another resolution planned in #95.
            None
        };

        // Process the requests.
        let mut ever_update_state = false;
        let response_batch = request_batch.iter().map(|ref queued_request| {
            let grpc_response = match Self::call_contract_fallible(
                contract,
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
            QueuedResponse { queued_request, grpc_response }
        }).collect();

        // Set state in consensus
        if let Some(encrypted_state) = encrypted_state_opt {
            if ever_update_state {
                let mut consensus_set_request = consensus::SetRequest::new();
                consensus_set_request.set_payload(encrypted_state.write_to_bytes()?);
                consensus_client
                    .set(grpc::RequestOptions::new(), consensus_set_request)
                    .wait()?;
            }
        }

        Ok(response_batch)
    }

    fn call_contract_batch(contract: &enclave::EkidenEnclave, request_batch: Vec<QueuedRequest>) {
        match Self::call_contract_batch_fallible(contract, &request_batch) {
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
}

impl Compute for ComputeServerImpl {
    fn call_contract(
        &self,
        _options: grpc::RequestOptions,
        rpc_request: CallContractRequest,
    ) -> grpc::SingleResponse<CallContractResponse> {
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

    fn ias_get_spid(
        &self,
        _options: grpc::RequestOptions,
        _request: IasGetSpidRequest,
    ) -> grpc::SingleResponse<IasGetSpidResponse> {
        let mut response = IasGetSpidResponse::new();

        response.set_spid(self.ias.get_spid().to_vec());

        return grpc::SingleResponse::completed(response);
    }

    fn ias_verify_quote(
        &self,
        _options: grpc::RequestOptions,
        request: IasVerifyQuoteRequest,
    ) -> grpc::SingleResponse<IasVerifyQuoteResponse> {
        let mut response = IasVerifyQuoteResponse::new();

        match self.ias
            .verify_quote(request.get_nonce(), request.get_quote())
        {
            Ok(report) => {
                response.set_status(match report.status {
                    200 => IasVerifyQuoteResponse_Status::SUCCESS,
                    400 => IasVerifyQuoteResponse_Status::ERROR_BAD_REQUEST,
                    401 => IasVerifyQuoteResponse_Status::ERROR_UNAUTHORIZED,
                    500 => IasVerifyQuoteResponse_Status::ERROR_INTERNAL_SERVER_ERROR,
                    503 => IasVerifyQuoteResponse_Status::ERROR_SERVICE_UNAVAILABLE,
                    _ => IasVerifyQuoteResponse_Status::ERROR_SERVICE_UNAVAILABLE,
                });

                response.set_body(report.body);
                response.set_signature(report.signature);
                response.set_certificates(report.certificates);
            }
            _ => {
                // Verification failed due to IAS communication error.
                response.set_status(IasVerifyQuoteResponse_Status::ERROR_SERVICE_UNAVAILABLE);
            }
        }

        return grpc::SingleResponse::completed(response);
    }
}
