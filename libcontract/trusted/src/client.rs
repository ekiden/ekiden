use sodalite;

use libcontract_common::api;
use libcontract_common::client::ClientEndpoint;
use libcontract_common::quote::{Quote, QUOTE_CONTEXT_SC_CLIENT_TO_CONTRACT};

use compute_client::Error;
use compute_client::backend::ContractClientBackend;

use super::dispatcher;
use super::quote::{create_report_data_for_public_key, get_quote, get_spid, verify_quote};

pub struct OcallContractClientBackend {
    /// Endpoint that the client is connecting to.
    endpoint: ClientEndpoint,
}

impl OcallContractClientBackend {
    /// Construct new OCALL contract client backend.
    pub fn new(endpoint: ClientEndpoint) -> Result<Self, Error> {
        Ok(OcallContractClientBackend { endpoint: endpoint })
    }
}

impl ContractClientBackend for OcallContractClientBackend {
    /// Call contract.
    fn call(&self, client_request: api::ClientRequest) -> Result<api::ClientResponse, Error> {
        Ok(dispatcher::untrusted_call_endpoint(
            &self.endpoint,
            client_request,
        )?)
    }

    /// Call contract with raw data.
    fn call_raw(&self, client_request: Vec<u8>) -> Result<Vec<u8>, Error> {
        Ok(dispatcher::untrusted_call_endpoint_raw(
            &self.endpoint,
            client_request,
        )?)
    }

    /// Get SPID that can be used to verify the quote later.
    fn get_spid(&self) -> Result<Vec<u8>, Error> {
        Ok(get_spid()?)
    }

    /// Verify quote via IAS.
    fn verify_quote(&self, quote: Vec<u8>) -> Result<Quote, Error> {
        Ok(verify_quote(quote)?)
    }

    /// Get quote of the local enclave for mutual attestation.
    fn get_quote(
        &self,
        spid: &Vec<u8>,
        nonce: &Vec<u8>,
        public_key: &sodalite::BoxPublicKey,
    ) -> Result<Vec<u8>, Error> {
        Ok(get_quote(
            spid.as_slice(),
            &QUOTE_CONTEXT_SC_CLIENT_TO_CONTRACT,
            create_report_data_for_public_key(nonce.as_slice(), &public_key)?,
        )?)
    }
}
