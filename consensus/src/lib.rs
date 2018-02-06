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
mod generated;
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
    pub grpc_port: u16,
}

pub fn run(config: &Config) -> Result<(), Box<Error>> {

    // Create a shared State object
    let state = Arc::new(Mutex::new(State::new()));

    // Create Tendermint proxy.
    let tendermint = TendermintProxy::new(&config.tendermint_host, config.tendermint_port);

    //// Start the Ekiden consensus gRPC server.
    //let port = value_t!(args, "grpc-port", u16).unwrap_or_else(|e| e.exit());
    //let mut rpc_server = grpc::ServerBuilder::new_plain();
    //rpc_server.http.set_port(port);
    //rpc_server.http.set_cpu_pool_threads(1);
    //rpc_server.add_service(ConsensusServer::new_service_def(ConsensusServerImpl::new(
    //    Arc::clone(&state),
    //    tendermint.get_channel(),
    //)));
    //let _server = rpc_server.build().expect("rpc_server");

    //// Start the Tendermint ABCI listener
    //let abci_listen_addr = SocketAddr::new(
    //    IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
    //    value_t!(args, "tendermint-abci-port", u16).unwrap_or_else(|e| e.exit()),
    //);
    //let mut app_server = TcpServer::new(AbciProto, abci_listen_addr);
    //app_server.threads(1);
    //app_server.serve(move || {
    //    Ok(AbciService {
    //        app: Box::new(ekidenmint::Ekidenmint::new(Arc::clone(&state))),
    //    })
    //});
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::generated::consensus;
    use super::generated::consensus_grpc;
    use super::generated::consensus_grpc::Consensus;
    use grpc;

    #[test]
    fn exercise1() {
        //let consensus_client =
        //    consensus_grpc::ConsensusClient::new_plain("localhost", 9002, Default::default())
        //        .unwrap();

        //// Set state to `helloworld`
        //let mut consensus_set_request = consensus::SetRequest::new();
        //consensus_set_request.set_payload(String::from("helloworld").into_bytes());
        //consensus_client
        //    .set(grpc::RequestOptions::new(), consensus_set_request)
        //    .wait()
        //    .unwrap();

        //let consensus_get_request = consensus::GetRequest::new();
        //let (_, consensus_get_response, _) = consensus_client
        //    .get(grpc::RequestOptions::new(), consensus_get_request)
        //    .wait()
        //    .unwrap();
        //assert_eq!(
        //    consensus_get_response.get_payload(),
        //    String::from("helloworld").as_bytes()
        //);

        //// Set state to `successor`
        //let mut consensus_set_request = consensus::SetRequest::new();
        //consensus_set_request.set_payload(String::from("successor").into_bytes());
        //consensus_client
        //    .set(grpc::RequestOptions::new(), consensus_set_request)
        //    .wait()
        //    .unwrap();

        //let consensus_get_request = consensus::GetRequest::new();
        //let (_, consensus_get_response, _) = consensus_client
        //    .get(grpc::RequestOptions::new(), consensus_get_request)
        //    .wait()
        //    .unwrap();
        //assert_eq!(
        //    consensus_get_response.get_payload(),
        //    String::from("successor").as_bytes()
        //);

        //// Set state to a sequence of all byte values
        //let mut scale: Vec<u8> = vec![0; 256];
        //for i in 0..256 {
        //    scale[i] = i as u8;
        //}

        //let mut consensus_set_request = consensus::SetRequest::new();
        //consensus_set_request.set_payload(scale.clone());
        //consensus_client
        //    .set(grpc::RequestOptions::new(), consensus_set_request)
        //    .wait()
        //    .unwrap();

        //let consensus_get_request = consensus::GetRequest::new();
        //let (_, consensus_get_response, _) = consensus_client
        //    .get(grpc::RequestOptions::new(), consensus_get_request)
        //    .wait()
        //    .unwrap();
        //assert_eq!(consensus_get_response.get_payload(), &scale[..]);
    }
}
