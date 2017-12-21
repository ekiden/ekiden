// TODO: license

#![crate_name = "guessenclave"]
#![crate_type = "staticlib"]

#![feature(prelude_import)]

#![no_std]

#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_rand as rand;

extern crate protobuf;
use protobuf::Message;

use core::cmp::Ordering;
use core::result::Result;
use rand::Rng;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

mod enclave_rpc;

#[derive(Copy, Clone)]
pub enum State {
    Guessing { secret: i32 },
    Guessed,
}

#[derive(Copy, Clone)]
pub enum GuessFeedback {
    Higher,
    Lower,
    Win,
}

#[no_mangle]
pub extern "C" fn guess_enclave_init(result: *mut Result<State, &'static str>) {
    std::backtrace::enable_backtrace("enclave.signed.so", std::backtrace::PrintFormat::Short).expect("Couldn't enable backtrace");
    let result = unsafe { &mut *result };
    let secret: i32 = rand::thread_rng().gen_range(1, 101);
    *result = Ok(State::Guessing { secret });
}

#[no_mangle]
pub extern "C" fn guess_enclave_guess(result: *mut Result<(State, GuessFeedback), &'static str>, state: *const State, guess: i32) {
    let result = unsafe { &mut *result };
    let state = unsafe { &*state };
    *result = if let State::Guessing { secret } = *state {
        match secret.cmp(&guess) {
            Ordering::Greater => Ok((*state, GuessFeedback::Higher)),
            Ordering::Less => Ok((*state, GuessFeedback::Lower)),
            Ordering::Equal => Ok((State::Guessed, GuessFeedback::Win)),
        }
    } else {
        Err("Invalid state")
    };
}

#[no_mangle]
pub extern "C" fn rpc_call(request_data: *const u8,
                           request_length: usize,
                           response_data: *const u8,
                           response_length: usize) {
    // Parse request message.
    let request = unsafe { std::slice::from_raw_parts(request_data, request_length) };
    let request: enclave_rpc::Request = protobuf::parse_from_bytes(request).expect("Failed to parse request");

    // TODO: Invoke given method.
    println!("Request method: {}", request.method);

    // Prepare response.
    let mut response = enclave_rpc::Response::new();
    response.set_code(enclave_rpc::Response_Code::SUCCESS);
    let response = response.write_to_bytes().expect("Failed to create response");
    // TODO: Send back response.
}
