use hyper::Error;
use std::io::{self, Write};
use std::sync::mpsc;
use futures::{Future, Stream};
use hyper;
use tokio_core::reactor::Core;

pub struct Tendermint {
  uri_prefix: String,
  core: Core,
  client: hyper::Client<hyper::client::HttpConnector>,
}

impl Tendermint {
  pub fn new(uri_prefix: String) -> Tendermint {
    let core = Core::new().unwrap();
    let client = hyper::Client::new(&core.handle());
    Tendermint {
      uri_prefix: uri_prefix,
      core: core,
      client: client, 
    }
  }

  fn helper(&mut self, path: String) -> Result<String, Error> {
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

  pub fn help(&mut self) -> Result<String, Error> {
    let uri = String::new() + &self.uri_prefix;
    self.helper(uri)
  }

  pub fn broadcast_tx_commit(&mut self, payload: Vec<u8>) -> Result<String, Error> {
    let payload_str = String::from_utf8(payload).unwrap();
    let uri = String::new() + &self.uri_prefix + 
      "/broadcast_tx_commit?tx=\"" + &payload_str + "\"";
    self.helper(uri)
  }

}
