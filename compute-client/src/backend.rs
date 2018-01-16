use grpc;

use protobuf;
use protobuf::Message;

use libcontract_common::api::{Request, Response};

use super::generated::compute_web3::CallContractRequest;
use super::generated::compute_web3_grpc::{Compute, ComputeClient};

use super::errors::Error;

/// Contract client backend.
pub trait ContractClientBackend {
    /// Call contract.
    fn call(&self, request: Request) -> Result<Response, Error>;
}

pub struct Web3ContractClientBackend {
    /// gRPC client instance.
    client: ComputeClient,
}

impl Web3ContractClientBackend {
    /// Construct new Web3 contract client backend.
    pub fn new(host: &str, port: u16) -> Result<Self, Error> {
        Ok(Web3ContractClientBackend {
            // TODO: Use TLS client.
            client: ComputeClient::new_plain(&host, port, Default::default()).unwrap(),
        })
    }
}

impl ContractClientBackend for Web3ContractClientBackend {
    fn call(&self, request: Request) -> Result<Response, Error> {
        let mut raw_request = CallContractRequest::new();
        raw_request.set_payload(request.write_to_bytes()?);

        // TODO: Handle errors.
        let (_, response, _) = self.client.call_contract(
            grpc::RequestOptions::new(),
            raw_request
        ).wait().unwrap();

        let response: Response = protobuf::parse_from_bytes(response.get_payload())?;

        Ok(response)
    }
}
