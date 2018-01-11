use grpc;

use protobuf;
use protobuf::{Message, MessageStatic};

use super::errors::Error;
use super::generated::compute_web3::{StatusRequest, CallContractRequest};
use super::generated::compute_web3_grpc::{Compute, ComputeClient};

/// Contract client.
pub struct ContractClient {
    client: ComputeClient,
}

pub struct ContractStatus {
    /// Contract name.
    pub contract: String,
    /// Contract version.
    pub version: String,
}

impl ContractClient {
    /// Constructs a new contract client.
    pub fn new(host: &str, port: u16) -> Self {
        ContractClient {
            // TODO: Use TLS client.
            client: ComputeClient::new_plain(&host, port, Default::default()).unwrap(),
        }
    }

    /// Calls a contract method.
    // TODO: have the compute node fetch and store the state
    pub fn call<Rq, Rs>(&self, method: &str, state: Vec<u8>, request: Rq) -> Result<(Vec<u8>, Rs), Error>
        where Rq: Message,
              Rs: Message + MessageStatic {

        let mut raw_request = CallContractRequest::new();
        raw_request.set_method(method.to_string());
        raw_request.set_payload(request.write_to_bytes().unwrap());
        raw_request.set_state(state);

        let (_, response, _) = self.client.call_contract(
            grpc::RequestOptions::new(),
            raw_request
        ).wait().unwrap();

        let state = response.get_state().to_vec();
        let response: Rs = protobuf::parse_from_bytes(response.get_payload()).unwrap();

        Ok((state, response))
    }

    /// Get compute node status.
    pub fn status(&self) -> Result<ContractStatus, Error> {
        let request = StatusRequest::new();
        let (_, mut response, _) = self.client.status(grpc::RequestOptions::new(), request).wait().unwrap();

        let mut contract = response.take_contract();

        Ok(ContractStatus {
            contract: contract.take_name(),
            version: contract.take_version(),
        })
    }
}
