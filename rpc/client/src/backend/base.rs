use futures::Future;
use sodalite;

use ekiden_common::error::Result;
use ekiden_enclave_common::quote::AttestationReport;
use ekiden_rpc_common::api;

use super::super::future::ClientFuture;

/// Contract client backend.
pub trait ContractClientBackend: Send {
    /// Spawn future using an executor.
    fn spawn<F: Future<Item = (), Error = ()> + Send + 'static>(&self, future: F);

    /// Call contract.
    fn call(&self, client_request: api::ClientRequest) -> ClientFuture<api::ClientResponse>;

    /// Call contract with raw data.
    fn call_raw(&self, request: Vec<u8>) -> ClientFuture<Vec<u8>>;

    /// Get attestation report of the local enclave for mutual attestation.
    ///
    /// This method can only be implemented by clients which are running in enclaves
    /// and should return an error otherwise.
    fn get_attestation_report(
        &self,
        public_key: &sodalite::BoxPublicKey,
    ) -> Result<AttestationReport>;
}
