#![feature(prelude_import)]

#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

extern crate libenclave_trusted;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

#[no_mangle]
pub extern "C" fn rpc_call(request_data: *const u8,
                           request_length: usize,
                           response_data: *mut u8,
                           response_capacity: usize,
                           response_length: *mut usize) {
    // TODO: Find a way to forward this automatically.
    libenclave_trusted::call(request_data, request_length, response_data, response_capacity, response_length);
}
