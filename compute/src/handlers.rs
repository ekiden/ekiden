/// Handlers for the endpoints available to be called from inside the enclave,
/// which are registered using RpcRouter.
use std::sync::Arc;

use futures::Future;
use tokio_core;

use protobuf::Message;

use ekiden_core_common::{Error, Result};
use ekiden_core_common::rpc::api;
use ekiden_core_common::rpc::api::services::*;
use ekiden_core_common::rpc::client::ClientEndpoint;
use ekiden_core_untrusted::impl_rpc_handler;
use ekiden_core_untrusted::rpc::router::Handler;

use ekiden_rpc_client::backend::{ContractClientBackend, Web3ContractClientBackend};

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
    fn get_spid(&self, _request: IasGetSpidRequest) -> Result<IasGetSpidResponse> {
        let mut response = IasGetSpidResponse::new();
        response.set_spid(self.ias.get_spid().to_vec());

        Ok(response)
    }

    /// Handle verify quote request.
    fn verify_quote(&self, request: IasVerifyQuoteRequest) -> Result<IasVerifyQuoteResponse> {
        match self.ias
            .verify_quote(request.get_nonce(), request.get_quote())
        {
            Ok(report) => {
                let mut response = IasVerifyQuoteResponse::new();
                let mut serialized_report = api::AttestationReport::new();
                serialized_report.set_body(report.body.clone());
                serialized_report.set_signature(report.signature.clone());
                serialized_report.set_certificates(report.certificates.clone());
                response.set_report(serialized_report);

                Ok(response)
            }
            _ => {
                // Verification failed due to IAS communication error.
                Err(Error::new("IAS communication error"))
            }
        }
    }
}

impl_rpc_handler! {
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
    /// Client backend.
    client: Web3ContractClientBackend,
}

impl ContractForwarder {
    pub fn new(
        endpoint: ClientEndpoint,
        reactor: tokio_core::reactor::Remote,
        host: String,
        port: u16,
    ) -> Self {
        ContractForwarder {
            endpoint: endpoint,
            client: Web3ContractClientBackend::new(reactor, &host, port).unwrap(),
        }
    }
}

impl Handler for ContractForwarder {
    /// Return a list of endpoints that the handler can handle.
    fn get_endpoints(&self) -> Vec<ClientEndpoint> {
        vec![self.endpoint.clone()]
    }

    /// Handle a request and return a response.
    fn handle(&self, _endpoint: &ClientEndpoint, request: Vec<u8>) -> Result<Vec<u8>> {
        // Currently all OCALLs are blocking so this handler is blocking as well.
        match self.client.call_raw(request).wait() {
            Ok(response) => Ok(response),
            _ => Err(Error::new("RPC call failed")),
        }
    }
}
