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

mod ekidenmint;
mod errors;
mod tendermint;
mod generated;
mod rpc;
mod state;

//use std::env;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;
use abci::server::{AbciProto, AbciService};
use tokio_proto::TcpServer;

use generated::storage_grpc::StorageServer;
use rpc::StorageServerImpl;
use state::State;

fn main() {
  println!("Ekiden Storage starting... ");
  // Create a shared State object
  let s = Arc::new(Mutex::new(State::new()));

  // Create Tendermint client.
  // We'll use a channel to funnel transactions to Tendermint client
  let tendermint_uri = String::from("http://localhost:46657");
  let (tx, rx) = mpsc::channel();
  let tx = Arc::new(Mutex::new(tx));
  thread::spawn(move || {
    let mut tendermint_client = tendermint::Tendermint::new(tendermint_uri);
    tendermint::proxy_broadcasts(&mut tendermint_client, rx);
  });

  // Start the gRPC server.
  let port = 9002;
  let mut rpc_server = grpc::ServerBuilder::new_plain();
  rpc_server.http.set_port(port);
  rpc_server.http.set_cpu_pool_threads(1);
  rpc_server.add_service(StorageServer::new_service_def(StorageServerImpl::new(Arc::clone(&s), Arc::clone(&tx))));
  let _server = rpc_server.build().expect("rpc_server");
  println!("Storage node listening at {}", port);

  // Start the Tendermint ABCI listener
  let abci_listen_addr = "127.0.0.1:46658".parse().unwrap();
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
    use grpc;
    use super::generated::storage;
    use super::generated::storage_grpc;
    use super::generated::storage_grpc::Storage;

    #[test]
    fn exercise1() {
        let storage_client = storage_grpc::StorageClient::new_plain("localhost", 9002, Default::default()).unwrap();

        // Set state to `helloworld`
        let mut storage_set_request = storage::SetRequest::new();
        storage_set_request.set_payload(String::from("helloworld").into_bytes());
        storage_client.set(grpc::RequestOptions::new(), storage_set_request).wait().unwrap();

        let storage_get_request = storage::GetRequest::new();
        let (_, storage_get_response, _) = storage_client.get(grpc::RequestOptions::new(), storage_get_request).wait().unwrap();
        assert_eq!(storage_get_response.get_payload(), String::from("helloworld").as_bytes());

        // Set state to `successor`
        let mut storage_set_request = storage::SetRequest::new();
        storage_set_request.set_payload(String::from("successor").into_bytes());
        storage_client.set(grpc::RequestOptions::new(), storage_set_request).wait().unwrap();

        let storage_get_request = storage::GetRequest::new();
        let (_, storage_get_response, _) = storage_client.get(grpc::RequestOptions::new(), storage_get_request).wait().unwrap();
        assert_eq!(storage_get_response.get_payload(), String::from("successor").as_bytes());

        // Set state to a sequence of all byte values
        let mut scale: Vec<u8> = vec![0; 256];
        for i in 0..256 {
            scale[i] = i as u8;
        }

        let mut storage_set_request = storage::SetRequest::new();
        storage_set_request.set_payload(scale.clone());
        storage_client.set(grpc::RequestOptions::new(), storage_set_request).wait().unwrap();

        let storage_get_request = storage::GetRequest::new();
        let (_, storage_get_response, _) = storage_client.get(grpc::RequestOptions::new(), storage_get_request).wait().unwrap();
        assert_eq!(storage_get_response.get_payload(), &scale[..]);
    }
}
