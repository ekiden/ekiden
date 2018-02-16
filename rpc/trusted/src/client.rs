use sodalite;

use futures::future::{self, Future};

use ekiden_enclave_common::error::Result;
use ekiden_enclave_common::quote::{AttestationReport, QUOTE_CONTEXT_SC};
use ekiden_rpc_client::ClientFuture;
use ekiden_rpc_client::backend::ContractClientBackend;
use ekiden_rpc_common::api;
use ekiden_rpc_common::client::ClientEndpoint;

use super::bridge;
use super::quote::create_attestation_report_for_public_key;

pub struct OcallContractClientBackend {
    /// Endpoint that the client is connecting to.
    endpoint: ClientEndpoint,
}

impl OcallContractClientBackend {
    /// Construct new OCALL contract client backend.
    pub fn new(endpoint: ClientEndpoint) -> Result<Self> {
        Ok(OcallContractClientBackend { endpoint: endpoint })
    }
}

impl ContractClientBackend for OcallContractClientBackend {
    /// Spawn future using an executor.
    fn spawn<F: Future + Send + 'static>(&self, _future: F) {
        panic!("Attempted to spawn future using OCALL backend");
    }

    /// Call contract.
    fn call(&self, client_request: api::ClientRequest) -> ClientFuture<api::ClientResponse> {
        let endpoint = self.endpoint.clone();

        Box::new(future::lazy(move || {
            Ok(bridge::untrusted_call_endpoint(&endpoint, client_request)?)
        }))
    }

    /// Call contract with raw data.
    fn call_raw(&self, client_request: Vec<u8>) -> ClientFuture<Vec<u8>> {
        let endpoint = self.endpoint.clone();

        Box::new(future::lazy(move || {
            Ok(bridge::untrusted_call_endpoint_raw(
                &endpoint,
                client_request,
            )?)
        }))
    }

    /// Get attestation report of the local enclave for mutual attestation.
    fn get_attestation_report(
        &self,
        public_key: &sodalite::BoxPublicKey,
    ) -> Result<AttestationReport> {
        Ok(create_attestation_report_for_public_key(
            &QUOTE_CONTEXT_SC,
            &[0; 16],
            &public_key,
        )?)
    }
}
