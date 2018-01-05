extern crate abci;
extern crate grpc;
extern crate protobuf;
extern crate tls_api;
extern crate tokio_proto;

mod ekidenmint;
mod generated;
mod rpc;
mod server;

//use std::env;
use abci::server::{ AbciProto, AbciService };
use tokio_proto::TcpServer;

use generated::storage_grpc::StorageRpcServer;
use rpc::StorageRpcServerImpl;

fn main() {
  println!("Ekiden Storage starting... ");
  // Start the gRPC server.
  let port = 9002;
  let mut rpc_server = grpc::ServerBuilder::new_plain();
  rpc_server.http.set_port(port);
  rpc_server.http.set_cpu_pool_threads(1);
  rpc_server.add_service(StorageRpcServer::new_service_def(StorageRpcServerImpl::new()));
  let _server = rpc_server.build().expect("rpc_server");
  println!("Storage node listening at {}", port);

  // Start the ABCI listener
  let abci_listen_addr = "127.0.0.1:46658".parse().unwrap();
  let app = ekidenmint::Ekidenmint::new();
  let app_server = TcpServer::new(AbciProto, abci_listen_addr);
  app_server.serve(move || {
    Ok(AbciService {
      app: Box::new(app.clone()),
    })
  });

}
