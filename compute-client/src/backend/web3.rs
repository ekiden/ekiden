use grpc;
use sodalite;

use protobuf;
use protobuf::Message;

use libcontract_common::api;
use libcontract_common::quote::AttestationReport;

use super::super::generated::compute_web3::CallContractRequest;
use super::super::generated::compute_web3_grpc::{Compute, ComputeClient};

use super::ContractClientBackend;
use super::super::errors::Error;

pub struct Web3ContractClientBackend {
    /// gRPC client instance.
    client: ComputeClient,
}

impl Web3ContractClientBackend {
    /// Construct new Web3 contract client backend.
    pub fn new(host: &str, port: u16) -> Result<Self, Error> {
        Ok(Web3ContractClientBackend {
            // TODO: Use TLS client.
            client: match ComputeClient::new_plain(&host, port, Default::default()) {
                Ok(client) => client,
                _ => return Err(Error::new("Failed to initialize gRPC client")),
            },
        })
    }
}

impl ContractClientBackend for Web3ContractClientBackend {
    /// Call contract.
    fn call(&self, client_request: api::ClientRequest) -> Result<api::ClientResponse, Error> {
        let client_response = self.call_raw(client_request.write_to_bytes()?)?;
        let client_response: api::ClientResponse = protobuf::parse_from_bytes(&client_response)?;

        Ok(client_response)
    }

    /// Call contract with raw data.
    fn call_raw(&self, client_request: Vec<u8>) -> Result<Vec<u8>, Error> {
        let mut rpc_request = CallContractRequest::new();
        rpc_request.set_payload(client_request);

        let mut rpc_response = match self.client
            .call_contract(grpc::RequestOptions::new(), rpc_request)
            .wait()
        {
            Ok((_, rpc_response, _)) => rpc_response,
            Err(error) => {
                return Err(Error::new(&format!(
                    "Failed to call contract (gRPC error: {:?})",
                    error
                )))
            }
        };

        Ok(rpc_response.take_payload())
    }

    /// Get attestation report of the local enclave for mutual attestation.
    fn get_attestation_report(
        &self,
        _public_key: &sodalite::BoxPublicKey,
    ) -> Result<AttestationReport, Error> {
        Err(Error::new(
            "This backend does not support mutual attestation",
        ))
    }
}
