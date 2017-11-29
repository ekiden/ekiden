extern crate tsp;

use std::env;
use std::thread;

mod ekidenmint;

fn main() {
  let args: Vec<String> = env::args().collect();
  //let connection_type: &str = &args[1];
  //let listen_addr: &str = &args[2];

  println!("Ekiden Storage starting... ");
  static APP: ekidenmint::Ekidenmint = ekidenmint::Ekidenmint;
  let app_server = tsp::server::new("127.0.0.1:46658".parse().unwrap(), &APP);

  loop {
    thread::park();
  }
}
