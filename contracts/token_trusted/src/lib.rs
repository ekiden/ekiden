#![crate_name = "token_trusted"]
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

use libcontract_trusted::generated::enclave_rpc;
use protobuf::Message;

mod token_contract;
mod generated;

#[no_mangle]
pub extern "C" fn rpc_call(request_data: *const u8,
                           request_length: usize,
                           response_data: *mut u8,
                           response_capacity: usize,
                           response_length: *mut usize) {
    // Parse request message.
    let request = unsafe { std::slice::from_raw_parts(request_data, request_length) };
    let request: enclave_rpc::Request = protobuf::parse_from_bytes(request).expect("Failed to parse request");

    // TODO: Invoke given method.
    println!("Request method: {}", request.method);

    // Prepare response.
    let mut response = enclave_rpc::Response::new();
    response.set_code(enclave_rpc::Response_Code::SUCCESS);
    let response = response.write_to_bytes().expect("Failed to create response");

    // Copy back response.
    if response.len() > response_capacity {
        panic!("Not enough space for response.");
    } else {
        unsafe {
            for i in 0..response.len() as isize {
                std::ptr::write(response_data.offset(i), response[i as usize]);
            }

            *response_length = response.len();
        };
    }
}
