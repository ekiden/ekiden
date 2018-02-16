use sodalite;

use futures::future::{self, Future};

use libcontract_common::api;
use libcontract_common::client::ClientEndpoint;
use libcontract_common::quote::{AttestationReport, QUOTE_CONTEXT_SC};

use compute_client::{ClientFuture, Error};
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
    /// Spawn future using an executor.
    fn spawn<F: Future<Item = (), Error = ()> + Send + 'static>(&self, _future: F) {
        panic!("Attempted to spawn future using OCALL backend");
    }

    /// Call contract.
    fn call(&self, client_request: api::ClientRequest) -> ClientFuture<api::ClientResponse> {
        let endpoint = self.endpoint.clone();

        Box::new(future::lazy(move || {
            Ok(dispatcher::untrusted_call_endpoint(
                &endpoint,
                client_request,
            )?)
        }))
    }

    /// Call contract with raw data.
    fn call_raw(&self, client_request: Vec<u8>) -> ClientFuture<Vec<u8>> {
        let endpoint = self.endpoint.clone();

        Box::new(future::lazy(move || {
            Ok(dispatcher::untrusted_call_endpoint_raw(
                &endpoint,
                client_request,
            )?)
        }))
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
