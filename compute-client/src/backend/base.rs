use libcontract_common::api;

use super::super::quote::Quote;
use super::super::errors::Error;

/// Contract client backend.
pub trait ContractClientBackend {
    /// Call contract.
    fn call(&self, request: api::Request) -> Result<api::Response, Error>;

    /// Get SPID that can be used to verify the quote later.
    fn get_spid(&self) -> Result<Vec<u8>, Error>;

    /// Verify quote via IAS.
    fn verify_quote(&self, quote: Vec<u8>) -> Result<Quote, Error>;
}
