use sgx_types::*;

use protobuf;
use protobuf::{Message, MessageStatic};

use libcontract_common::{api, random};
use libcontract_common::client::ClientEndpoint;

use compute_client::{Error, Quote};
use compute_client::backend::ContractClientBackend;

use super::untrusted;

pub struct OcallContractClientBackend {
    /// Endpoint that the client is connecting to.
    endpoint: ClientEndpoint,
}

impl OcallContractClientBackend {
    /// Construct new OCALL contract client backend.
    pub fn new(endpoint: ClientEndpoint) -> Result<Self, Error> {
        Ok(OcallContractClientBackend { endpoint: endpoint })
    }

    /// Perform an RPC call against a given endpoint.
    pub fn call_endpoint<Rq, Rs>(&self, endpoint: &ClientEndpoint, request: Rq) -> Result<Rs, Error>
    where
        Rq: Message,
        Rs: Message + MessageStatic,
    {
        Ok(protobuf::parse_from_bytes(
            &self.call_raw(request.write_to_bytes()?)?,
        )?)
    }

    /// Perform a raw RPC call against a given endpoint.
    fn call_endpoint_raw(
        &self,
        endpoint: &ClientEndpoint,
        request: Vec<u8>,
    ) -> Result<Vec<u8>, Error> {
        // Maximum size of serialized response is 16K.
        let mut response: Vec<u8> = Vec::with_capacity(16 * 1024);

        let mut response_length = 0;
        let status = unsafe {
            untrusted::untrusted_rpc_call(
                endpoint.as_u16(),
                request.as_ptr() as *const u8,
                request.len(),
                response.as_mut_ptr() as *mut u8,
                response.capacity(),
                &mut response_length,
            )
        };

        match status {
            sgx_status_t::SGX_SUCCESS => {}
            _ => {
                return Err(Error::new(
                    "Outgoing enclave RPC call failed (error during OCALL)",
                ));
            }
        }

        unsafe {
            response.set_len(response_length);
        }

        Ok(response)
    }
}

impl ContractClientBackend for OcallContractClientBackend {
    /// Call contract.
    fn call(&self, client_request: api::ClientRequest) -> Result<api::ClientResponse, Error> {
        Ok(self.call_endpoint(&self.endpoint, client_request)?)
    }

    /// Call contract with raw data.
    fn call_raw(&self, client_request: Vec<u8>) -> Result<Vec<u8>, Error> {
        Ok(self.call_endpoint_raw(&self.endpoint, client_request)?)
    }

    /// Get SPID that can be used to verify the quote later.
    fn get_spid(&self) -> Result<Vec<u8>, Error> {
        let request = api::services::IasGetSpidRequest::new();
        let mut response: api::services::IasGetSpidResponse =
            self.call_endpoint(&ClientEndpoint::IASProxyGetSpid, request)?;

        Ok(response.take_spid())
    }

    /// Verify quote via IAS.
    fn verify_quote(&self, quote: Vec<u8>) -> Result<Quote, Error> {
        let decoded = Quote::decode(&quote)?;

        let mut request = api::services::IasVerifyQuoteRequest::new();
        request.set_quote(quote);

        // Generate random nonce.
        let mut nonce = vec![0u8; 16];
        random::get_random_bytes(&mut nonce);
        request.set_nonce(nonce.clone());

        let response: api::services::IasVerifyQuoteResponse =
            self.call_endpoint(&ClientEndpoint::IASProxyVerifyQuote, request)?;

        // TODO: Check response, verify signatures, verify nonce etc.

        Ok(decoded)
    }
}
