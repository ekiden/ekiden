use std::sync::{Arc, Mutex};
use grpc;
//use protobuf;

use generated::storage::{GetRequest, GetResponse, SetRequest, SetResponse};
use generated::storage_grpc::Storage;
use state::State;
use tendermint::Tendermint;

pub struct StorageServerImpl {
    server: Arc<Mutex<State>>,
    tendermint: Tendermint,
}

impl StorageServerImpl {
  pub fn new(server: Arc<Mutex<State>>) -> StorageServerImpl {
    StorageServerImpl {
      server: server,
      tendermint: Tendermint::new(),
    }
  }
}

impl Storage for StorageServerImpl {
  fn get(&self, _options: grpc::RequestOptions, _req: GetRequest) -> grpc::SingleResponse<GetResponse> {
    let s = self.server.lock().unwrap();
    match s.get_latest() {
      Some(val) => {
	let mut response = GetResponse::new();
      	response.set_payload(val);
	grpc::SingleResponse::completed(response)
      }
      None => {
	grpc::SingleResponse::err(grpc::Error::Other(""))
      }
    }
  }

  fn set(&self, _options: grpc::RequestOptions, req: SetRequest) -> grpc::SingleResponse<SetResponse> {
    let payload = req.get_payload();

    return grpc::SingleResponse::completed(SetResponse::new());
  }
}

