#![feature(prelude_import)]
#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate libcontract_trusted;
extern crate libcontract_common;

#[macro_use]
extern crate dummy_api;

extern crate protobuf;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

use dummy_api::{HelloWorldRequest, HelloWorldResponse};
use protobuf::well_known_types::Empty;

use libcontract_common::ContractError;

create_enclave_api!();

fn hello_world(_state: Empty, request: HelloWorldRequest) -> Result<(Empty, HelloWorldResponse), ContractError> {
    println!("hello world called");

    let mut response = HelloWorldResponse::new();
    response.set_world(format!("enclave says {}", request.hello));

    Ok((Empty::new(), response))
}
