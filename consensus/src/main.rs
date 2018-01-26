extern crate abci;
extern crate futures;
extern crate grpc;
extern crate hyper;
extern crate protobuf;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tls_api;
extern crate tokio_core;
extern crate tokio_proto;

#[macro_use]
extern crate clap;

mod ekidenmint;
mod errors;
mod tendermint;
mod generated;
mod rpc;
mod state;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::{Arc, Mutex};
use std::sync::mpsc;
use std::thread;

use abci::server::{AbciProto, AbciService};
use clap::{App, Arg};
use tokio_proto::TcpServer;

use generated::consensus_grpc::ConsensusServer;
use rpc::ConsensusServerImpl;
use state::State;

fn main() {
    let matches = App::new("Ekiden Compute Node")
        .version("0.1.0")
        .about("Ekident consensus node")
        .arg(
            Arg::with_name("tendermint-host")
                .long("tendermint-host")
                .takes_value(true)
                .default_value("localhost"),
        )
        .arg(
            Arg::with_name("tendermint-port")
                .long("tendermint-port")
                .takes_value(true)
                .default_value("46657"),
        )
        .arg(
            Arg::with_name("tendermint-abci-port")
                .long("tendermint-abci-port")
                .takes_value(true)
                .default_value("46658"),
        )
        .arg(
            Arg::with_name("grpc-port")
                .long("grpc-port")
                .takes_value(true)
                .default_value("9002"),
        )
        .get_matches();

    println!("Ekiden Consensus starting... ");

    // Create a shared State object
    let s = Arc::new(Mutex::new(State::new()));

    // Create Tendermint client.
    // We'll use a channel to funnel transactions to Tendermint client
    let tendermint_uri = format!(
        "http://{}:{}",
        matches.value_of("tendermint-host").unwrap().to_string(),
        value_t!(matches, "tendermint-port", u16).unwrap_or_else(|e| e.exit())
    );
    let (tx, rx) = mpsc::channel();
    let tx = Arc::new(Mutex::new(tx));
    thread::spawn(move || {
        let mut tendermint_client = tendermint::Tendermint::new(tendermint_uri);
        tendermint::proxy_broadcasts(&mut tendermint_client, rx);
    });

    // Start the gRPC server.
    let port = value_t!(matches, "grpc-port", u16).unwrap_or_else(|e| e.exit());
    let mut rpc_server = grpc::ServerBuilder::new_plain();
    rpc_server.http.set_port(port);
    rpc_server.http.set_cpu_pool_threads(1);
    rpc_server.add_service(ConsensusServer::new_service_def(ConsensusServerImpl::new(
        Arc::clone(&s),
        Arc::clone(&tx),
    )));
    let _server = rpc_server.build().expect("rpc_server");
    println!("Consensus node listening at {}", port);

    // Start the Tendermint ABCI listener
    let abci_listen_addr = SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        value_t!(matches, "tendermint-abci-port", u16).unwrap_or_else(|e| e.exit()),
    );
    let mut app_server = TcpServer::new(AbciProto, abci_listen_addr);
    app_server.threads(1);
    app_server.serve(move || {
        Ok(AbciService {
            app: Box::new(ekidenmint::Ekidenmint::new(Arc::clone(&s))),
        })
    });
}

#[cfg(test)]
mod tests {
    use super::generated::consensus;
    use super::generated::consensus_grpc;
    use super::generated::consensus_grpc::Consensus;
    use grpc;

    #[test]
    fn exercise1() {
        let consensus_client =
            consensus_grpc::ConsensusClient::new_plain("localhost", 9002, Default::default())
                .unwrap();

        // Set state to `helloworld`
        let mut consensus_set_request = consensus::SetRequest::new();
        consensus_set_request.set_payload(String::from("helloworld").into_bytes());
        consensus_client
            .set(grpc::RequestOptions::new(), consensus_set_request)
            .wait()
            .unwrap();

        let consensus_get_request = consensus::GetRequest::new();
        let (_, consensus_get_response, _) = consensus_client
            .get(grpc::RequestOptions::new(), consensus_get_request)
            .wait()
            .unwrap();
        assert_eq!(
            consensus_get_response.get_payload(),
            String::from("helloworld").as_bytes()
        );

        // Set state to `successor`
        let mut consensus_set_request = consensus::SetRequest::new();
        consensus_set_request.set_payload(String::from("successor").into_bytes());
        consensus_client
            .set(grpc::RequestOptions::new(), consensus_set_request)
            .wait()
            .unwrap();

        let consensus_get_request = consensus::GetRequest::new();
        let (_, consensus_get_response, _) = consensus_client
            .get(grpc::RequestOptions::new(), consensus_get_request)
            .wait()
            .unwrap();
        assert_eq!(
            consensus_get_response.get_payload(),
            String::from("successor").as_bytes()
        );

        // Set state to a sequence of all byte values
        let mut scale: Vec<u8> = vec![0; 256];
        for i in 0..256 {
            scale[i] = i as u8;
        }

        let mut consensus_set_request = consensus::SetRequest::new();
        consensus_set_request.set_payload(scale.clone());
        consensus_client
            .set(grpc::RequestOptions::new(), consensus_set_request)
            .wait()
            .unwrap();

        let consensus_get_request = consensus::GetRequest::new();
        let (_, consensus_get_response, _) = consensus_client
            .get(grpc::RequestOptions::new(), consensus_get_request)
            .wait()
            .unwrap();
        assert_eq!(consensus_get_response.get_payload(), &scale[..]);
    }
}
