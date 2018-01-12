use grpc;
use thread_local::ThreadLocal;

use libcontract_untrusted::enclave;

use generated::compute_web3::{StatusRequest, StatusResponse, CallContractRequest, CallContractResponse};
use generated::compute_web3_grpc::Compute;

pub struct ComputeServerImpl {
    // Filename of the enclave implementing the contract.
    contract_filename: String,
    // Contract running in an enclave.
    contract: ThreadLocal<enclave::EkidenEnclave>,
}

impl ComputeServerImpl {
    /// Create new compute server instance.
    pub fn new(contract_filename: &str) -> Self {
        ComputeServerImpl {
            contract_filename: contract_filename.to_string(),
            contract: ThreadLocal::new(),
        }
    }

    /// Get thread-local instance of the contract.
    fn get_contract(&self) -> &enclave::EkidenEnclave {
        self.contract.get_or(|| {
            // TODO: Handle contract initialization errors.
            let contract = enclave::EkidenEnclave::new(&self.contract_filename).unwrap();

            // Initialize contract.
            // TODO: Support passing non-zero sealed state.
            contract.initialize(vec![]).expect("Failed to initialize contract");

            Box::new(contract)
        })
    }
}

impl Compute for ComputeServerImpl {
    fn status(&self, _options: grpc::RequestOptions, _request: StatusRequest) -> grpc::SingleResponse<StatusResponse> {
        // Get contract metadata.
        let metadata = match self.get_contract().get_metadata() {
            Ok(metadata) => metadata,
            Err(_) => return grpc::SingleResponse::err(grpc::Error::Other("Failed to get metadata"))
        };

        let mut response = StatusResponse::new();
        {
            let contract = response.mut_contract();
            contract.set_name(metadata.get_name().to_string());
            contract.set_version(metadata.get_version().to_string());
        }

        return grpc::SingleResponse::completed(response);
    }

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
}
