use std::sync::{Arc, Mutex};
use grpc;
//use protobuf;

use generated::storage::{GetRequest, GetResponse, SetRequest, SetResponse};
use generated::storage_grpc::StorageRpc;
use server::StorageServer;

pub struct StorageRpcServerImpl {
    server: Arc<Mutex<StorageServer>>,
}

impl StorageRpcServerImpl {
    pub fn new(server: Arc<Mutex<StorageServer>>) -> StorageRpcServerImpl {
	StorageRpcServerImpl {
	  server: server,
	}
    }
}

impl StorageRpc for StorageRpcServerImpl {
  fn get(&self, _options: grpc::RequestOptions, req: GetRequest) -> grpc::SingleResponse<GetResponse> {
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
    let mut response = SetResponse::new();
    return grpc::SingleResponse::completed(response);
  }
}

