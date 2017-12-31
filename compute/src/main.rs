extern crate libcontract_untrusted;

use std::env;
use libcontract_untrusted::enclave;
use libcontract_untrusted::generated::enclave_rpc;

fn main() {
  let enclave_filename = env::args().nth(1).expect("Usage: compute enclave_filename");
  // Create a new ekiden enclave from the given library.
  let e = enclave::EkidenEnclave::new(&enclave_filename).unwrap();

  // Fire off an RPC.
  let mut request = enclave_rpc::Request::new();
  request.set_method(String::from("hello_world"));
  let response = e.call(&request).unwrap();
  println!("Response status={:?}", response.code);

  // Destroy the enclave.
  e.destroy();
}

