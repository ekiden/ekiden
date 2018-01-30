/// Handlers for the endpoints available to be called from inside the enclave,
/// which are registered using RpcRouter.
use std::sync::Arc;

use protobuf::Message;

use libcontract_common::api::services::*;
use libcontract_common::client::ClientEndpoint;
use libcontract_untrusted::errors::Error;
use libcontract_untrusted::impl_handler;
use libcontract_untrusted::router::Handler;

use compute_client::backend::{ContractClientBackend, Web3ContractClientBackend};

use super::ias::IAS;

/// IAS proxy endpoints.
pub struct IASProxy {
    /// Shared IAS interface.
    ias: Arc<IAS>,
}

impl IASProxy {
    pub fn new(ias: Arc<IAS>) -> Self {
        IASProxy { ias: ias }
    }

    /// Handle get SPID request.
    fn get_spid(&self, _request: IasGetSpidRequest) -> Result<IasGetSpidResponse, Error> {
        let mut response = IasGetSpidResponse::new();
        response.set_spid(self.ias.get_spid().to_vec());

        Ok(response)
    }

    /// Handle verify quote request.
    fn verify_quote(
        &self,
        request: IasVerifyQuoteRequest,
    ) -> Result<IasVerifyQuoteResponse, Error> {
        match self.ias
            .verify_quote(request.get_nonce(), request.get_quote())
        {
            Ok(report) => {
                let mut response = IasVerifyQuoteResponse::new();
                response.set_report(report.serialize());

                Ok(response)
            }
            _ => {
                // Verification failed due to IAS communication error.
                Err(Error::OtherError("IAS communication error".to_string()))
            }
        }
    }
}

impl_handler! {
    for IASProxy {
        IASProxyGetSpid => get_spid,
        IASProxyVerifyQuote => verify_quote,
    }
}

/// Generic contract endpoint.
///
/// This endpoint can be used to forward requests to an arbitrary destination
/// contract, defined by the `hostname` and `port` of the compute node that is
/// running the contract.
pub struct ContractForwarder {
    /// Client endpoint identifier.
    endpoint: ClientEndpoint,
    /// Target contract hostname.
    host: String,
    /// Target contract port.
    port: u16,
}

impl ContractForwarder {
    pub fn new(endpoint: ClientEndpoint, host: String, port: u16) -> Self {
        ContractForwarder {
            endpoint: endpoint,
            host: host,
            port: port,
        }
    }
}

impl Handler for ContractForwarder {
    /// Return a list of endpoints that the handler can handle.
    fn get_endpoints(&self) -> Vec<ClientEndpoint> {
        vec![self.endpoint.clone()]
    }

    /// Handle a request and return a response.
    fn handle(&self, _endpoint: &ClientEndpoint, request: Vec<u8>) -> Result<Vec<u8>, Error> {
        let client = match Web3ContractClientBackend::new(&self.host, self.port) {
            Ok(client) => client,
            _ => return Err(Error::RpcRouterCallFailed),
        };

        match client.call_raw(request) {
            Ok(response) => Ok(response),
            _ => Err(Error::RpcRouterCallFailed),
        }
    }
}
