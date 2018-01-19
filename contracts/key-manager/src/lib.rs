#![feature(prelude_import)]
#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

extern crate libcontract_common;
#[macro_use]
extern crate libcontract_trusted;

#[macro_use]
extern crate key_manager_api;

extern crate protobuf;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

use key_manager_api::{HelloWorldRequest, HelloWorldResponse};

use libcontract_common::ContractError;

create_enclave_api!();

fn hello_world(request: HelloWorldRequest) -> Result<HelloWorldResponse, ContractError> {
    println!("hello world called");

    let mut response = HelloWorldResponse::new();
    response.set_world(format!("enclave says {}", request.hello));

    Ok(response)
}
