use grpc;
//use protobuf;

use generated::storage::{GetRequest, GetResponse, SetRequest, SetResponse};
use generated::storage_grpc::Storage;

pub struct StorageServerImpl {
}

impl StorageServerImpl {
  pub fn new() -> Self {
    StorageServerImpl {
    }
  }
}

impl Storage for StorageServerImpl {
  fn get(&self, _options: grpc::RequestOptions, req: GetRequest) -> grpc::SingleResponse<GetResponse> {
    let mut response = GetResponse::new();
    return grpc::SingleResponse::completed(response);
  }

  fn set(&self, _options: grpc::RequestOptions, req: SetRequest) -> grpc::SingleResponse<SetResponse> {
    let mut response = SetResponse::new();
    return grpc::SingleResponse::completed(response);
  }
}
