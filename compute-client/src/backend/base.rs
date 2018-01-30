use sodalite;

use libcontract_common::api;
use libcontract_common::quote::AttestationReport;

use super::super::errors::Error;

/// Contract client backend.
pub trait ContractClientBackend {
    /// Call contract.
    fn call(&self, client_request: api::ClientRequest) -> Result<api::ClientResponse, Error>;

    /// Call contract with raw data.
    fn call_raw(&self, request: Vec<u8>) -> Result<Vec<u8>, Error>;

    /// Get attestation report of the local enclave for mutual attestation.
    ///
    /// This method can only be implemented by clients which are running in enclaves
    /// and should return an error otherwise.
    fn get_attestation_report(
        &self,
        public_key: &sodalite::BoxPublicKey,
    ) -> Result<AttestationReport, Error>;
}
