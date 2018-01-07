use std::sync::{Arc, Mutex, mpsc};
use grpc;
//use protobuf;

use generated::storage::{GetRequest, GetResponse, SetRequest, SetResponse};
use generated::storage_grpc::Storage;
use state::State;

pub struct StorageServerImpl {
  state: Arc<Mutex<State>>,
  tx: Arc<Mutex<mpsc::Sender<Vec<u8>>>>,
}

impl StorageServerImpl {
  pub fn new(state: Arc<Mutex<State>>, tx: Arc<Mutex<mpsc::Sender<Vec<u8>>>>) -> StorageServerImpl {
    StorageServerImpl {
      state: state,
      tx: tx,
    }
  }
}

impl Storage for StorageServerImpl {
  fn get(&self, _options: grpc::RequestOptions, _req: GetRequest) -> grpc::SingleResponse<GetResponse> {
    let s = self.state.lock().unwrap();
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

