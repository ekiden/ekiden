use std::sync::mpsc;
use futures::{Future, Stream};
use hyper;
use serde_json;
use tokio_core;

use errors::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct JsonRpcResult<T> {
  pub jsonrpc: String,
  pub id: String,
  pub result: T
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BroadcastTxCommit {
  pub check_tx: CheckTx,
  pub deliver_tx: DeliverTx,
  pub hash: String,
  pub height: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BroadcastTx {
  pub code: i32,
  pub data: String,
  pub log: String,
  pub hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CheckTx {
  pub code: i32,
  pub data: String,
  pub log: String,
  pub gas: String,
  pub fee: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeliverTx {
  pub code: i32,
  pub data: String,
  pub log: String,
  pub tags: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Commit {
  pub canonical: bool,
  pub commit: CommitContent,
  pub header: CommitHeader,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitContent {
  pub blockID: BlockId,
  pub precommits: Vec<PreCommit>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PreCommit {
  pub block_id: BlockId,
  pub height: u64,
  pub round: u64,
  pub signature: Signature,
  //pub type: i32, // `type` is a keyword
  pub validator_address: String,
  pub validator_index: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Signature {
  pub data: String,
  //pub type: String, //`type` is a keyword
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CommitHeader {
  pub app_hash: String,
  pub chain_id: String,
  pub data_hash: String,
  pub height: u64,
  pub last_block_id: BlockId,
  pub last_commit_hash: String,
  pub num_txs: u64,
  pub time: String,
  pub validators_hash: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockId {
  pub hash: String,
  pub parts: BlockIdParts,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BlockIdParts {
  pub hash: String,
  pub total: u64,
}

pub struct BroadcastRequest {
  pub chan: mpsc::Sender<Result<JsonRpcResult<BroadcastTxCommit>, Error>>,
  pub payload: Vec<u8>,
}

pub fn proxy_broadcasts(client: &mut Tendermint, rx: mpsc::Receiver<BroadcastRequest>) {
  for req in rx {
    let result = client.broadcast_tx_commit(req.payload);
    req.chan.send(result).unwrap();
  }
}

pub struct Tendermint {
  uri_prefix: String,
  core: tokio_core::reactor::Core,
  client: hyper::Client<hyper::client::HttpConnector>,
}

impl Tendermint {
  pub fn new(uri_prefix: String) -> Tendermint {
    let core = tokio_core::reactor::Core::new().unwrap();
    let client = hyper::Client::new(&core.handle());
    Tendermint {
      uri_prefix: uri_prefix,
      core: core,
      client: client, 
    }
  }

  fn helper(&mut self, path: String) -> Result<String, Error> {
    println!("{}", path);
    let uri = path.parse()?;
    let work = self.client.get(uri).and_then(|res| {
      //println!("Response: {}", res.status());
      res.body().concat2()
    });
    let body = self.core.run(work)?; // Returns error if not reachable
    let body_vec = body.to_vec(); 
    let body_str = String::from_utf8(body_vec)?;
    Ok(body_str)
  }

  pub fn help(&mut self) -> Result<serde_json::Value, Error> {
    let uri = String::new() + &self.uri_prefix;
    let resp_str = self.helper(uri)?;
    let result: serde_json::Value = serde_json::from_str(&resp_str)?;
    Ok(result)
  }

  pub fn broadcast_tx_commit(&mut self, payload: Vec<u8>) -> Result<JsonRpcResult<BroadcastTxCommit>, Error> {
    let payload_str = String::from_utf8(payload)?;
    let uri = String::new() + &self.uri_prefix +
      "/broadcast_tx_commit?tx=\"" + &payload_str + "\"";
    let resp_str = self.helper(uri)?;
    let result: JsonRpcResult<BroadcastTxCommit> = serde_json::from_str(&resp_str)?;
    Ok(result)
  }

  pub fn broadcast_tx_async(&mut self, payload: Vec<u8>) -> Result<JsonRpcResult<BroadcastTx>, Error> {
    let payload_str = String::from_utf8(payload)?;
    let uri = String::new() + &self.uri_prefix +
      "/broadcast_tx_async?tx=\"" + &payload_str + "\"";
    let resp_str = self.helper(uri)?;
    let result: JsonRpcResult<BroadcastTx> = serde_json::from_str(&resp_str)?;
    Ok(result)
  }

  pub fn broadcast_tx_sync(&mut self, payload: Vec<u8>) -> Result<JsonRpcResult<BroadcastTx>, Error> {
    let payload_str = String::from_utf8(payload)?;
    let uri = String::new() + &self.uri_prefix +
      "/broadcast_tx_sync?tx=\"" + &payload_str + "\"";
    let resp_str = self.helper(uri)?;
    let result: JsonRpcResult<BroadcastTx> = serde_json::from_str(&resp_str)?;
    Ok(result)
  }

  pub fn commit(&mut self, height: u64) -> Result<serde_json::Value, Error> {
    let height_str = height.to_string();
    let uri = String::new() + &self.uri_prefix +
      "/commit?height=" + &height_str;
    let resp_str = self.helper(uri)?;
    let result: serde_json::Value = serde_json::from_str(&resp_str)?;
    Ok(result)
  }

}
