#![feature(prelude_import)]
#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate libcontract_trusted;
extern crate protobuf;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

mod generated;

use generated::dummy::{HelloWorldRequest, HelloWorldResponse};

// Create enclave.
create_enclave! {
    rpc hello_world(HelloWorldRequest) -> HelloWorldResponse;
}

fn hello_world(request: HelloWorldRequest) -> Result<HelloWorldResponse, ()> {
    println!("hello world called");

    let mut response = HelloWorldResponse::new();
    response.set_world(format!("enclave says {}", request.hello));

    Ok(response)
}
