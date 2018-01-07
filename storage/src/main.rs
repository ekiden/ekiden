extern crate abci;
extern crate futures;
extern crate grpc;
extern crate hyper;
extern crate protobuf;
extern crate tls_api;
extern crate tokio_core;
extern crate tokio_proto;

mod ekidenmint;
mod tendermint;
mod generated;
mod rpc;
mod state;

//use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use abci::server::{AbciProto, AbciService};
use tokio_proto::TcpServer;

use generated::storage_grpc::StorageServer;
use rpc::StorageServerImpl;
use state::State;

fn main() {
  println!("Ekiden Storage starting... ");
  // Create a shared State object
  let s = Arc::new(Mutex::new(State::new()));

  // Create Tendermint client
  let tendermint_uri = String::from("http://localhost:46657");
  thread::spawn(move || {
    thread::sleep(Duration::from_secs(3));
    let mut tendermint_client = tendermint::Tendermint::new(tendermint_uri);
    let arg = String::from("helloworld").into_bytes();
    let output = tendermint_client.broadcast_tx_commit(arg).unwrap();
    println!("\nMain output: {}", output);
  });

  // Start the gRPC server.
  let port = 9002;
  let mut rpc_server = grpc::ServerBuilder::new_plain();
  rpc_server.http.set_port(port);
  rpc_server.http.set_cpu_pool_threads(1);
  rpc_server.add_service(StorageServer::new_service_def(StorageServerImpl::new(Arc::clone(&s))));
  let _server = rpc_server.build().expect("rpc_server");
  println!("Storage node listening at {}", port);

  // Start the Tendermint ABCI listener
  let abci_listen_addr = "127.0.0.1:46658".parse().unwrap();
  let app = ekidenmint::Ekidenmint::new(Arc::clone(&s));
  let app_server = TcpServer::new(AbciProto, abci_listen_addr);
  app_server.serve(move || {
    Ok(AbciService {
      app: Box::new(app.clone()),
    })
  });

}
