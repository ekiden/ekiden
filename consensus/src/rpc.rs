use std;
use std::sync::{mpsc, Arc, Mutex};

use grpc;
use protobuf;
use protobuf::Message;

use super::generated::consensus;
use super::generated::consensus_grpc::Consensus;
use super::state;

use super::tendermint::BroadcastRequest;

pub struct ConsensusServerImpl {
    state: Arc<Mutex<state::State>>,
    // TODO: Clone the sender for each thread and store it in thread-local storage.
    broadcast_channel: Mutex<mpsc::Sender<BroadcastRequest>>,
}

impl ConsensusServerImpl {
    pub fn new(
        state: Arc<Mutex<state::State>>,
        broadcast_channel: mpsc::Sender<BroadcastRequest>,
    ) -> ConsensusServerImpl {
        ConsensusServerImpl {
            state: state,
            broadcast_channel: Mutex::new(broadcast_channel),
        }
    }

    fn replace_fallible(
        &self,
        payload: Vec<u8>,
    ) -> Result<consensus::ReplaceResponse, Box<std::error::Error>> {
        let mut stored = consensus::StoredTx::new();
        stored.set_replace(payload);
        let stored_bytes = stored.write_to_bytes()?;

        // check attestation - early reject
        state::State::check_tx(&stored_bytes)?;

        // Create a one-shot channel for response.
        let (tx, rx) = mpsc::channel();
        let req = BroadcastRequest {
            response: tx,
            payload: stored_bytes,
        };

        let broadcast_channel = self.broadcast_channel.lock().unwrap();
        broadcast_channel.send(req).unwrap();
        rx.recv().unwrap()?;

        Ok(consensus::ReplaceResponse::new())
    }

    fn add_diff_fallible(
        &self,
        payload: Vec<u8>,
    ) -> Result<consensus::AddDiffResponse, Box<std::error::Error>> {
        let mut stored = consensus::StoredTx::new();
        stored.set_diff(payload);
        let stored_bytes = stored.write_to_bytes()?;

        // check attestation - early reject
        state::State::check_tx(&stored_bytes)?;

        // Create a one-shot channel for response.
        let (tx, rx) = mpsc::channel();
        let req = BroadcastRequest {
            response: tx,
            payload: stored_bytes,
        };

        let broadcast_channel = self.broadcast_channel.lock().unwrap();
        broadcast_channel.send(req).unwrap();
        rx.recv().unwrap()?;

        Ok(consensus::AddDiffResponse::new())
    }
}

impl Consensus for ConsensusServerImpl {
    fn get(
        &self,
        _options: grpc::RequestOptions,
        _req: consensus::GetRequest,
    ) -> grpc::SingleResponse<consensus::GetResponse> {
        let s = self.state.lock().unwrap();
        match s.everything {
            Some(ref si) => {
                let mut response = consensus::GetResponse::new();
                {
                    let mut checkpoint = response.mut_checkpoint();
                    checkpoint.set_payload(si.checkpoint.clone());
                    checkpoint.set_height(si.checkpoint_height);
                }
                response.set_diffs(protobuf::RepeatedField::from_vec(si.diffs.clone()));
                grpc::SingleResponse::completed(response)
            }
            None => grpc::SingleResponse::err(grpc::Error::Other("State not initialized.")),
        }
    }

    fn get_diffs(
        &self,
        _options: grpc::RequestOptions,
        req: consensus::GetDiffsRequest,
    ) -> grpc::SingleResponse<consensus::GetDiffsResponse> {
        let s = self.state.lock().unwrap();
        match s.everything {
            Some(ref si) => {
                let mut response = consensus::GetDiffsResponse::new();
                if si.checkpoint_height > req.get_since_height() {
                    // We don't have diffs going back far enough.
                    {
                        let mut checkpoint = response.mut_checkpoint();
                        checkpoint.set_payload(si.checkpoint.clone());
                        checkpoint.set_height(si.checkpoint_height);
                    }
                    response.set_diffs(protobuf::RepeatedField::from_vec(si.diffs.clone()));
                } else {
                    let num_known = req.get_since_height() - si.checkpoint_height;
                    response.set_diffs(protobuf::RepeatedField::from_vec(
                        si.diffs[num_known as usize..].to_vec(),
                    ));
                }
                grpc::SingleResponse::completed(response)
            }
            None => grpc::SingleResponse::err(grpc::Error::Other("State not initialized.")),
        }
    }

    fn replace(
        &self,
        _options: grpc::RequestOptions,
        req: consensus::ReplaceRequest,
    ) -> grpc::SingleResponse<consensus::ReplaceResponse> {
        match self.replace_fallible(req.get_payload().to_vec()) {
            Ok(res) => grpc::SingleResponse::completed(res),
            Err(e) => grpc::SingleResponse::err(grpc::Error::Panic(e.description().to_owned())),
        }
    }

    fn add_diff(
        &self,
        _options: grpc::RequestOptions,
        req: consensus::AddDiffRequest,
    ) -> grpc::SingleResponse<consensus::AddDiffResponse> {
        match self.add_diff_fallible(req.get_payload().to_vec()) {
            Ok(res) => grpc::SingleResponse::completed(res),
            Err(e) => grpc::SingleResponse::err(grpc::Error::Panic(e.description().to_owned())),
        }
    }
}
