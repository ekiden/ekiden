use std::io::{self, Write};
use std::sync::mpsc;
use futures::{Future, Stream};
use hyper;
use serde_json;
use tokio_core;

use errors::Error;

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

  fn helper(&mut self, path: String) -> Result<serde_json::Value, Error> {
    println!("{}", path);
    let uri = path.parse()?;
    let work = self.client.get(uri).and_then(|res| {
      //println!("Response: {}", res.status());
      res.body().concat2()
    });
    let body = self.core.run(work)?; // Returns error if not reachable
    let body_vec = body.to_vec(); 
    let body_str = String::from_utf8(body_vec)?;
    let body_json: serde_json::Value = serde_json::from_str(&body_str)?;
    Ok(body_json)
  }

  pub fn help(&mut self) -> Result<serde_json::Value, Error> {
    let uri = String::new() + &self.uri_prefix;
    self.helper(uri)
  }

  pub fn broadcast_tx_commit(&mut self, payload: Vec<u8>) -> Result<serde_json::Value, Error> {
    let payload_str = String::from_utf8(payload).unwrap();
    let uri = String::new() + &self.uri_prefix +
      "/broadcast_tx_commit?tx=\"" + &payload_str + "\"";
    self.helper(uri)
  }

  pub fn commit(&mut self, height: u64) -> Result<serde_json::Value, Error> {
    let height_str = height.to_string();
    let uri = String::new() + &self.uri_prefix +
      "/commit?height=" + &height_str;
    self.helper(uri)
  }

}
