use libcontract_common::api;

use super::super::errors::Error;
use super::super::quote::Quote;

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
}
