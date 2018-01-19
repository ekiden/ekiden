use grpc;
use thread_local::ThreadLocal;

use std::sync::{Arc, Mutex};
use std::fmt::Write;

use libcontract_untrusted::enclave;

use generated::compute_web3::{CallContractRequest, CallContractResponse, IasGetSpidRequest, IasGetSpidResponse,
                              IasVerifyQuoteRequest, IasVerifyQuoteResponse};
use generated::compute_web3_grpc::Compute;

use super::ias::{IAS, IASConfiguration};

pub struct ComputeServerImpl {
    // Filename of the enclave implementing the contract.
    contract_filename: String,
    // Contract running in an enclave.
    contract: ThreadLocal<enclave::EkidenEnclave>,
    // IAS service.
    ias: IAS,
}

impl ComputeServerImpl {
    /// Create new compute server instance.
    pub fn new(contract_filename: &str, ias: IASConfiguration) -> Self {
        ComputeServerImpl {
            contract_filename: contract_filename.to_string(),
            contract: ThreadLocal::new(),
            ias: IAS::new(ias).unwrap(),
        }
    }

    /// Get thread-local instance of the contract.
    fn get_contract(&self) -> &enclave::EkidenEnclave {
        self.contract.get_or(|| {
            // TODO: Handle contract initialization errors.
            let contract = enclave::EkidenEnclave::new(&self.contract_filename).unwrap();

            // Initialize contract.
            // TODO: Support contract restore.
            let response = contract.initialize().expect("Failed to initialize contract");

            // Show contract MRENCLAVE in hex format.
            let mut mr_enclave = String::new();
            for &byte in response.get_mr_enclave() {
                write!(&mut mr_enclave, "{:02x}", byte).unwrap();
            }

            println!("Loaded contract with MRENCLAVE: {}", mr_enclave);

            Box::new(contract)
        })
    }
}

impl Compute for ComputeServerImpl {
    fn call_contract(&self, _options: grpc::RequestOptions, request: CallContractRequest)
                     -> grpc::SingleResponse<CallContractResponse> {

        let raw_response = match self.get_contract().call_raw(&request.get_payload().to_vec()) {
            Ok(response) => response,
            Err(_) => return grpc::SingleResponse::err(grpc::Error::Other("Failed to call contract"))
        };

        let mut response = CallContractResponse::new();
        response.set_payload(raw_response);

        return grpc::SingleResponse::completed(response);
    }

    fn ias_get_spid(&self, _options: grpc::RequestOptions, _request: IasGetSpidRequest)
                    -> grpc::SingleResponse<IasGetSpidResponse> {

        let mut response = IasGetSpidResponse::new();

        response.set_spid(self.ias.get_spid().to_vec());

        return grpc::SingleResponse::completed(response);
    }

    fn ias_verify_quote(&self, _options: grpc::RequestOptions, request: IasVerifyQuoteRequest)
                        -> grpc::SingleResponse<IasVerifyQuoteResponse> {

        let mut response = IasVerifyQuoteResponse::new();

        match self.ias.verify_quote(request.get_nonce(), request.get_quote()) {
            Ok(report) => {
                // Verification successful.
                response.set_success(true);
                response.set_body(report.body);
                response.set_signature(report.signature);
                response.set_certificates(report.certificates);
            },
            _ => {
                // Verification failed.
                response.set_success(false);
            }
        }

        return grpc::SingleResponse::completed(response);
    }
}
