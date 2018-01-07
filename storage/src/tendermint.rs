use hyper::Error;
use std::io::{self, Write};
use futures::{Future, Stream};
use hyper;
use tokio_core::reactor::Core;

pub struct Tendermint {
  core: Core,
  client: hyper::Client<hyper::client::HttpConnector>,
}

impl Tendermint {
  pub fn new() -> Tendermint {
    let core = Core::new().unwrap();
    let client = hyper::Client::new(&core.handle());
    Tendermint {
      core: core,
      client: client, 
    }
  }

  pub fn broadcast_tx_commit(&mut self, payload: Vec<u8>) -> Result<(), Error> {
    let uri = "http://httpbin.org/ip".parse()?;
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
