extern crate abci;
extern crate futures;
extern crate grpc;
extern crate hyper;
extern crate protobuf;
extern crate tls_api;
extern crate tokio_core;
extern crate tokio_proto;

mod ekidenmint;
mod generated;
mod rpc;
mod server;

//use std::env;
use std::sync::{Arc, Mutex};
use abci::server::{AbciProto, AbciService};
use tokio_proto::TcpServer;

use generated::storage_grpc::StorageRpcServer;
use rpc::StorageRpcServerImpl;
use server::StorageServer;

fn main() {
  println!("Ekiden Storage starting... ");
  // Create a shared StorageServer object
  let s = Arc::new(Mutex::new(StorageServer::new()));

  let abci_listen_addr = "127.0.0.1:46658".parse().unwrap();
  let app = ekidenmint::Ekidenmint::new(Arc::clone(&s));
  let app_server = TcpServer::new(AbciProto, abci_listen_addr);

  // Start the gRPC server.
  let port = 9002;
  let mut rpc_server = grpc::ServerBuilder::new_plain();
  rpc_server.http.set_port(port);
  rpc_server.http.set_cpu_pool_threads(1);
  rpc_server.add_service(StorageRpcServer::new_service_def(StorageRpcServerImpl::new(Arc::clone(&s))));
  let _server = rpc_server.build().expect("rpc_server");
  println!("Storage node listening at {}", port);

  // Start the Tendermint ABCI listener
  app_server.serve(move || {
    Ok(AbciService {
      app: Box::new(app.clone()),
    })
  });

}
