extern crate abci;
extern crate tokio_proto;

mod ekidenmint;

use std::env;
use tokio_proto::TcpServer;

use abci::server::{ AbciProto, AbciService };

fn main() {
  let args: Vec<String> = env::args().collect();
  //let connection_type: &str = &args[1];
  //let listen_addr: &str = &args[2];
  let listen_addr = "127.0.0.1:46658".parse().unwrap();

  println!("Ekiden Storage starting... ");
  let app = ekidenmint::Ekidenmint::new();
  let app_server = TcpServer::new(AbciProto, listen_addr);
  app_server.serve(move || {
    Ok(AbciService {
      app: Box::new(app.clone()),
    })
  });

}
