use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

use grpc;

use super::generated::tendermint::{RequestBroadcastTx, ResponseBroadcastTx};
use super::generated::tendermint_grpc::{BroadcastAPI, BroadcastAPIClient};

/// Broadcast request that can be sent via the proxy.
pub struct BroadcastRequest {
    /// Raw broadcast payload.
    pub payload: Vec<u8>,
    /// Channel for sending the response.
    pub response: Sender<Result<ResponseBroadcastTx, grpc::Error>>,
}

/// Proxy that runs the tendermint client in a separate thread.
pub struct TendermintProxy {
    /// Broadcast channel.
    sender: Sender<BroadcastRequest>,
}

impl TendermintProxy {
    /// Create a new Tendermint proxy instance.
    pub fn new(host: &str, port: u16) -> Self {
        // Create new channel.
        let (sender, receiver) = channel();

        let proxy = TendermintProxy { sender: sender };

        proxy.start(host, port, receiver);

        proxy
    }

    /// Get the channel that can be used for sending requests.
    pub fn get_channel(&self) -> Sender<BroadcastRequest> {
        self.sender.clone()
    }

    /// Start the proxy worker thread.
    fn start(&self, host: &str, port: u16, queue: Receiver<BroadcastRequest>) {
        let client = BroadcastAPIClient::new_plain(&host, port, Default::default()).unwrap();

        thread::spawn(move || {
            // Process requests in queue.
            for request in queue {
                let mut broadcast_request = RequestBroadcastTx::new();
                broadcast_request.set_tx(request.payload);

                let response = match client
                    .broadcast_tx(grpc::RequestOptions::new(), broadcast_request)
                    .wait()
                {
                    Ok((_, response, _)) => Ok(response),
                    Err(error) => Err(error),
                };

                request.response.send(response).unwrap();
            }
        });
    }
}
