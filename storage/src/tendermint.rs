use hyper::Error;
use std::io::{self, Write};
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

  pub fn broadcast_tx_commit(&mut self, payload: Vec<u8>) -> Result<(), Error> {
    let uri = self.uri_prefix.parse()?;
    let work = self.client.get(uri).and_then(|res| {
      println!("Response: {}", res.status());
      res.body().for_each(|chunk| {
	io::stdout()
	  .write_all(&chunk)
	  .map_err(From::from)
      })
    });
    self.core.run(work)?;
    Ok(())
  }

}
