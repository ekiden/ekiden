extern crate libcontract_untrusted;
extern crate protobuf;

use std::env;
use libcontract_untrusted::enclave;

mod generated;

use generated::dummy::{HelloWorldRequest, HelloWorldResponse};

fn main() {
    let enclave_filename = env::args().nth(1).expect("Usage: compute enclave_filename");

    // Create a new ekiden enclave from the given library.
    let e = enclave::EkidenEnclave::new(&enclave_filename).unwrap();

    // Fire off an RPC.
    let mut request = HelloWorldRequest::new();
    request.set_hello(String::from("hello rpc!"));
    let response: HelloWorldResponse = e.call("hello_world", &request).unwrap();
    println!("Response={:?}", response.world);

    // Destroy the enclave.
    e.destroy();
}
