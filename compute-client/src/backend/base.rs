use sodalite;

use libcontract_common::api;
use libcontract_common::quote::Quote;

use super::super::errors::Error;

/// Contract client backend.
pub trait ContractClientBackend {
    /// Call contract.
    fn call(&self, client_request: api::ClientRequest) -> Result<api::ClientResponse, Error>;

    /// Call contract with raw data.
    fn call_raw(&self, request: Vec<u8>) -> Result<Vec<u8>, Error>;

    /// Get SPID that can be used to verify the quote later.
    fn get_spid(&self) -> Result<Vec<u8>, Error>;

    /// Verify quote via IAS.
    fn verify_quote(&self, quote: Vec<u8>) -> Result<Quote, Error>;

    /// Get quote of the local enclave for mutual attestation.
    ///
    /// This method can only be implemented by clients, which are running in enclaves
    /// and should return an error otherwise.
    fn get_quote(
        &self,
        spid: &Vec<u8>,
        nonce: &Vec<u8>,
        public_key: &sodalite::BoxPublicKey,
    ) -> Result<Vec<u8>, Error>;
}
