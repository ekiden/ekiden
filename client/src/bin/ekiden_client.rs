extern crate grpc;
extern crate futures;
extern crate ekiden_web3;

use ekiden_web3::generated::ekiden_web3_grpc::*;
use ekiden_web3::generated::ekiden_web3::*;

use std::env;

fn main() {
    let name = env::args().nth(1).map(|s| s.to_owned()).unwrap_or_else(|| "world".to_owned());

    let client = EkidenClient::new_plain("localhost", 9001, Default::default()).unwrap();

    let mut req = HelloRequest::new();
    req.set_name(name);

    let resp = client.say_hello(grpc::RequestOptions::new(), req);

    println!("{:?}", resp.wait());
}
