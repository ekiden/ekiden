use grpc;
use thread_local::ThreadLocal;

use libcontract_untrusted::enclave;

use generated::compute_web3::{CallContractRequest, CallContractResponse};
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
            // TODO: Support contract restore.
            contract.initialize().expect("Failed to initialize contract");

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
}
