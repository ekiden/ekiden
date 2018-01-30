use sodalite;

use libcontract_common::api;
use libcontract_common::client::ClientEndpoint;
use libcontract_common::quote::{AttestationReport, QUOTE_CONTEXT_SC};

use compute_client::Error;
use compute_client::backend::ContractClientBackend;

use super::dispatcher;
use super::quote::create_attestation_report_for_public_key;

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

    /// Get attestation report of the local enclave for mutual attestation.
    fn get_attestation_report(
        &self,
        public_key: &sodalite::BoxPublicKey,
    ) -> Result<AttestationReport, Error> {
        Ok(create_attestation_report_for_public_key(
            &QUOTE_CONTEXT_SC,
            &[0; 16],
            &public_key,
        )?)
    }
}
