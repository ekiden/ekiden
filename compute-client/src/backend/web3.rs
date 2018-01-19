use grpc;
use rand::{OsRng, Rng};

use protobuf;
use protobuf::Message;

use libcontract_common::api;

use super::super::generated::compute_web3::{CallContractRequest, IasGetSpidRequest,
                                            IasVerifyQuoteRequest};
use super::super::generated::compute_web3_grpc::{Compute, ComputeClient};

use super::ContractClientBackend;
use super::super::errors::Error;
use super::super::quote::Quote;

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
            _ => return Err(Error::new("Failed to call contract")),
        };

        Ok(rpc_response.take_payload())
    }

    /// Get SPID that can be used to verify the quote later.
    fn get_spid(&self) -> Result<Vec<u8>, Error> {
        // TODO: Cache SPID.

        let mut response = match self.client
            .ias_get_spid(grpc::RequestOptions::new(), IasGetSpidRequest::new())
            .wait()
        {
            Ok((_, response, _)) => response,
            _ => return Err(Error::new("Failed to get SPID from compute node")),
        };

        Ok(response.take_spid())
    }

    /// Verify quote via IAS.
    fn verify_quote(&self, quote: Vec<u8>) -> Result<Quote, Error> {
        let decoded = Quote::decode(&quote)?;

        let mut request = IasVerifyQuoteRequest::new();
        request.set_quote(quote);

        // Generate random nonce.
        let mut nonce = vec![0u8; 16];
        OsRng::new()?.fill_bytes(&mut nonce);
        request.set_nonce(nonce.clone());

        let response = match self.client
            .ias_verify_quote(grpc::RequestOptions::new(), request)
            .wait()
        {
            Ok((_, response, _)) => response,
            _ => return Err(Error::new("Failed to verify quote")),
        };

        // TODO: Check response, verify signatures, verify nonce etc.

        Ok(decoded)
    }
}
