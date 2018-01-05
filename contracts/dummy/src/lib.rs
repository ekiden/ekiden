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

use generated::api::{HelloWorldRequest, HelloWorldResponse};
use libcontract_trusted::common::contract_error::ContractError;

// Create enclave.
create_enclave! {
    metadata {
        name = "dummy";
        version = "0.1.0";
    }

    rpc hello_world(HelloWorldRequest) -> HelloWorldResponse;
}

fn hello_world(request: HelloWorldRequest) -> Result<HelloWorldResponse, ContractError> {
    println!("hello world called");

    let mut response = HelloWorldResponse::new();
    response.set_world(format!("enclave says {}", request.hello));

    Ok(response)
}
