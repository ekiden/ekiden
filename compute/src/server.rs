use grpc;
use protobuf;
use protobuf::Message;
use std;
use thread_local::ThreadLocal;

use std::fmt::Write;
use std::sync::Arc;

use libcontract_common;
use libcontract_untrusted::enclave;

use generated::compute_web3::{CallContractRequest, CallContractResponse, IasGetSpidRequest,
                              IasGetSpidResponse, IasVerifyQuoteRequest, IasVerifyQuoteResponse,
                              IasVerifyQuoteResponse_Status};
use generated::compute_web3_grpc::Compute;
use generated::consensus;
use generated::consensus_grpc;
use generated::consensus_grpc::Consensus;

use super::ias::IAS;

pub struct ComputeServerImpl {
    // Filename of the enclave implementing the contract.
    contract_filename: String,
    // Contract running in an enclave.
    contract: ThreadLocal<enclave::EkidenEnclave>,
    // IAS service.
    ias: Arc<IAS>,
}

impl ComputeServerImpl {
    /// Create new compute server instance.
    pub fn new(contract_filename: &str, ias: Arc<IAS>) -> Self {
        ComputeServerImpl {
            contract_filename: contract_filename.to_string(),
            contract: ThreadLocal::new(),
            ias: ias,
        }
    }

    /// Get thread-local instance of the contract.
    fn get_contract(&self) -> &enclave::EkidenEnclave {
        self.contract.get_or(|| {
            // TODO: Handle contract initialization errors.
            let contract = enclave::EkidenEnclave::new(&self.contract_filename).unwrap();

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

            Box::new(contract)
        })
    }

    fn call_contract_fallible(
        &self,
        rpc_request: CallContractRequest,
    ) -> Result<CallContractResponse, Box<std::error::Error>> {
        // Connect to consensus node
        // TODO: Let client select consensus node.
        // TODO: Use TLS client.
        let consensus_client =
            consensus_grpc::ConsensusClient::new_plain("localhost", 9002, Default::default())?;

        //
        let mut enclave_request = libcontract_common::api::EnclaveRequest::new();

        // Get state from consensus
        let consensus_result = consensus_client
            .get(grpc::RequestOptions::new(), consensus::GetRequest::new())
            .wait();
        if let Ok((_, consensus_get_response, _)) = consensus_result {
            let encrypted_state = protobuf::parse_from_bytes(consensus_get_response.get_payload())?;
            enclave_request.set_encrypted_state(encrypted_state);
        };
        // We should bail if there was an error other than the consensus not being initialized.
        // But don't go fixing this. There's another resolution planned in #95.

        //
        let client_request: libcontract_common::api::ClientRequest =
            protobuf::parse_from_bytes(rpc_request.get_payload())?;
        enclave_request.set_client_request(client_request);

        let enclave_request_bytes = enclave_request.write_to_bytes()?;
        let enclave_response_bytes = self.get_contract().call_raw(enclave_request_bytes)?;

        let enclave_response: libcontract_common::api::EnclaveResponse =
            protobuf::parse_from_bytes(&enclave_response_bytes)?;

        // Set state in consensus
        if enclave_response.has_encrypted_state() {
            let new_encrypted_state = enclave_response.get_encrypted_state();
            let mut consensus_set_request = consensus::SetRequest::new();
            consensus_set_request.set_payload(new_encrypted_state.write_to_bytes()?);
            consensus_client
                .set(grpc::RequestOptions::new(), consensus_set_request)
                .wait()?;
        }

        //
        let client_response = enclave_response.get_client_response();

        let mut rpc_response = CallContractResponse::new();
        rpc_response.set_payload(client_response.write_to_bytes()?);
        Ok(rpc_response)
    }
}

impl Compute for ComputeServerImpl {
    fn call_contract(
        &self,
        _options: grpc::RequestOptions,
        rpc_request: CallContractRequest,
    ) -> grpc::SingleResponse<CallContractResponse> {
        match self.call_contract_fallible(rpc_request) {
            Ok(rpc_response) => grpc::SingleResponse::completed(rpc_response),
            Err(_) => {
                return grpc::SingleResponse::err(grpc::Error::Other("Failed to call contract"))
            }
        }
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
