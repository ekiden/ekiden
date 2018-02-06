extern crate abci;
extern crate futures;
extern crate grpc;
extern crate hyper;
extern crate protobuf;
extern crate tls_api;
extern crate tokio_core;
extern crate tokio_proto;

mod ekidenmint;
mod errors;
mod tendermint;
pub mod generated;
mod rpc;
mod state;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};

use abci::server::{AbciProto, AbciService};
use tokio_proto::TcpServer;

use errors::Error;
use generated::consensus_grpc::ConsensusServer;
use rpc::ConsensusServerImpl;
use state::State;
use tendermint::TendermintProxy;

pub struct Config {
    pub tendermint_host: String,
    pub tendermint_port: u16,
    pub tendermint_abci_port: u16,
    pub grpc_port: u16,
}

pub fn run(config: &Config) -> Result<(), Box<Error>> {
    // Create a shared State object
    let state = Arc::new(Mutex::new(State::new()));

    // Create Tendermint proxy.
    let tendermint = TendermintProxy::new(&config.tendermint_host, config.tendermint_port);

    // Start the Ekiden consensus gRPC server.
    let mut rpc_server = grpc::ServerBuilder::new_plain();
    rpc_server.http.set_port(config.grpc_port);
    rpc_server.http.set_cpu_pool_threads(1);
    rpc_server.add_service(ConsensusServer::new_service_def(ConsensusServerImpl::new(
        Arc::clone(&state),
        tendermint.get_channel(),
    )));
    let _server = rpc_server.build().expect("rpc_server");

    // Start the Tendermint ABCI listener
    let abci_listen_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        config.tendermint_abci_port,
    );
    let mut app_server = TcpServer::new(AbciProto, abci_listen_addr);
    app_server.threads(1);
    app_server.serve(move || {
        Ok(AbciService {
            app: Box::new(ekidenmint::Ekidenmint::new(Arc::clone(&state))),
        })
    });
    Ok(())
}

#[cfg(test)]
mod tests {
    //use super::generated::consensus;

    #[test]
    fn empty() {
        assert_eq!(8, 8)
    }
}
