use grpc;
use protobuf;
use protobuf::Message;

use libcontract_untrusted::enclave;
use libcontract_common::api;

use generated::compute_web3::{StatusRequest, StatusResponse, CallContractRequest, CallContractResponse};
use generated::compute_web3_grpc::Compute;

pub struct ComputeServerImpl {
    // Contract running in an enclave.
    contract: enclave::EkidenEnclave,
}

impl ComputeServerImpl {
    pub fn new(contract: enclave::EkidenEnclave) -> Self {
        ComputeServerImpl {
            contract: contract,
        }
    }
}

impl Compute for ComputeServerImpl {
    fn status(&self, _options: grpc::RequestOptions, _request: StatusRequest) -> grpc::SingleResponse<StatusResponse> {
        // Get contract metadata.
        let metadata = match self.contract.get_metadata() {
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

        let mut enclave_request = api::Request::new();
        enclave_request.set_method(request.get_method().to_string());
        enclave_request.set_payload(request.get_payload().to_vec());
        let encrypted_state_raw = request.get_encrypted_state();
        if encrypted_state_raw.len() != 0 {
            let encrypted_state = protobuf::parse_from_bytes(encrypted_state_raw).unwrap();
            enclave_request.set_encrypted_state(encrypted_state);
        }

        let mut raw_response = match self.contract.call_raw(&enclave_request) {
            Ok(response) => response,
            Err(_) => return grpc::SingleResponse::err(grpc::Error::Other("Failed to call contract"))
        };

        // Validate response code.
        match raw_response.get_code() {
            api::Response_Code::SUCCESS => {},
            _ => {
                // Deserialize error.
                let mut error: api::Error = match protobuf::parse_from_bytes(&raw_response.take_payload()) {
                    Ok(error) => error,
                    _ => return grpc::SingleResponse::err(grpc::Error::Other("Unknown error"))
                };

                return grpc::SingleResponse::err(grpc::Error::Panic(error.take_message()));
            }
        };

        let mut response = CallContractResponse::new();
        response.set_payload(raw_response.take_payload());
        response.set_encrypted_state(raw_response.get_encrypted_state().write_to_bytes().unwrap());

        return grpc::SingleResponse::completed(response);
    }
}
