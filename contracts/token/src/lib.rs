#![crate_name = "token"]
#![crate_type = "staticlib"]

#![no_std]
//#[allow(unused)]
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::v1::*;

#[macro_use]
extern crate sgx_tstd as std;
extern crate protobuf;
extern crate libcontract_trusted;

mod token_contract;
mod generated;

#[no_mangle]
pub extern "C" fn rpc_call(request_data: *const u8,
    request_length: usize,
    response_data: *mut u8,
    response_capacity: usize,
    response_length: *mut usize) {
  libcontract_trusted::rpc::call(request_data, request_length, response_data, response_capacity, response_length);
}

