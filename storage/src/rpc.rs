use std::sync::{Arc, Mutex, mpsc};
use grpc;

use generated::storage::{GetRequest, GetResponse, SetRequest, SetResponse};
use generated::storage_grpc::Storage;
use state::State;
use tendermint::BroadcastRequest;

pub struct StorageServerImpl {
  state: Arc<Mutex<State>>,
  tx: Arc<Mutex<mpsc::Sender<BroadcastRequest>>>,
}

impl StorageServerImpl {
  pub fn new(state: Arc<Mutex<State>>, tx: Arc<Mutex<mpsc::Sender<BroadcastRequest>>>) -> StorageServerImpl {
    StorageServerImpl {
      state: state,
      tx: tx,
    }
  }
}

impl Storage for StorageServerImpl {

  // Handle `get` requests to retrieve latest state
  fn get(&self, _options: grpc::RequestOptions, _req: GetRequest) -> grpc::SingleResponse<GetResponse> {
    let s = self.state.lock().unwrap();
    match s.get_latest() {
      Some(val) => {
        let mut response = GetResponse::new();
        response.set_payload(val);
        grpc::SingleResponse::completed(response)
      }
      None => {
        grpc::SingleResponse::err(grpc::Error::Other("State not initialized."))
      }
    }
  }

  // Set the next state as latest
  fn set(&self, _options: grpc::RequestOptions, req: SetRequest) -> grpc::SingleResponse<SetResponse> {
    let payload = req.get_payload();

    // check attestation - early reject
    match State::check_tx(payload) {
      Ok(_) => {
        // Create a one-shot channel for response
        let (tx, rx) = mpsc::channel();
        let req = BroadcastRequest {
          chan: tx,
          payload: payload.to_vec(),
        };
        let broadcast_channel = self.tx.lock().unwrap();
        broadcast_channel.send(req).unwrap();
        match rx.recv().unwrap() {
          Ok(_result) => grpc::SingleResponse::completed(SetResponse::new()),
          Err(_error) => grpc::SingleResponse::err(grpc::Error::Other("Error forwarding to Tendermint")),
        }
      },
      Err(_error) => grpc::SingleResponse::err(grpc::Error::Other("Invalid payload fails check_tx")),
    }
  }
}