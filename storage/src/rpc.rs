use grpc;
//use protobuf;

use generated::storage::{GetRequest, GetResponse, SetRequest, SetResponse};
use generated::storage_grpc::StorageRpc;

pub struct StorageRpcServerImpl {
}

impl StorageRpcServerImpl {
  pub fn new() -> Self {
    StorageRpcServerImpl {
    }
  }
}

impl StorageRpc for StorageRpcServerImpl {
  fn get(&self, _options: grpc::RequestOptions, req: GetRequest) -> grpc::SingleResponse<GetResponse> {
    let mut response = GetResponse::new();
    return grpc::SingleResponse::completed(response);
  }

  fn set(&self, _options: grpc::RequestOptions, req: SetRequest) -> grpc::SingleResponse<SetResponse> {
    let mut response = SetResponse::new();
    return grpc::SingleResponse::completed(response);
  }
}

