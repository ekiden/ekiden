use std;
use grpc;
use protobuf;
use protobuf::Message;
use thread_local::ThreadLocal;

use libcontract_common;
use libcontract_untrusted::enclave;

use generated::compute_web3::{StatusRequest, StatusResponse, CallContractRequest, CallContractResponse};
use generated::compute_web3_grpc::Compute;
use generated::storage;
use generated::storage_grpc;
use generated::storage_grpc::Storage;

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

    fn call_contract_fallible(&self, rpc_request: CallContractRequest) -> Result<CallContractResponse, Box<std::error::Error>> {
        // Connect to storage node
        // TODO: Let client select storage node.
        // TODO: Use TLS client.
        let storage_client = storage_grpc::StorageClient::new_plain("localhost", 9002, Default::default())?;

        //
        let mut enclave_request = libcontract_common::api::EnclaveRequest::new();

        // Get state from storage
        let storage_result = storage_client.get(grpc::RequestOptions::new(), storage::GetRequest::new()).wait();
        if let Ok((_, storage_get_response, _)) = storage_result {
            let encrypted_state = protobuf::parse_from_bytes(storage_get_response.get_payload())?;
            enclave_request.set_encrypted_state(encrypted_state);
        };
        // We should bail if there was an error other than the storage not being initialized.
        // But don't go fixing this. There's another resolution planned in #95.

        //
        let client_request: libcontract_common::api::ClientRequest = protobuf::parse_from_bytes(rpc_request.get_payload())?;
        enclave_request.set_client_request(client_request);

        let enclave_request_bytes = enclave_request.write_to_bytes()?;
        let enclave_response_bytes = self.get_contract().call_raw(&enclave_request_bytes)?;

        let enclave_response: libcontract_common::api::EnclaveResponse = protobuf::parse_from_bytes(&enclave_response_bytes)?;

        // Set state in storage
        if enclave_response.has_encrypted_state() {
            let new_encrypted_state = enclave_response.get_encrypted_state();
            let mut storage_set_request = storage::SetRequest::new();
            storage_set_request.set_payload(new_encrypted_state.write_to_bytes()?);
            storage_client.set(grpc::RequestOptions::new(), storage_set_request).wait()?;
        }

        //
        let client_response = enclave_response.get_client_response();

        let mut rpc_response = CallContractResponse::new();
        rpc_response.set_payload(client_response.write_to_bytes()?);
        Ok(rpc_response)
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

    fn call_contract(&self, _options: grpc::RequestOptions, rpc_request: CallContractRequest)
        -> grpc::SingleResponse<CallContractResponse> {
        match self.call_contract_fallible(rpc_request) {
            Ok(rpc_response) => grpc::SingleResponse::completed(rpc_response),
            Err(_) => return grpc::SingleResponse::err(grpc::Error::Other("Failed to call contract")),
        }
    }
}
