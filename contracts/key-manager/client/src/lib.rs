#![feature(prelude_import)]
#![feature(use_extern_macros)]
#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate lazy_static;

extern crate ekiden_enclave_common;
extern crate ekiden_rpc_client;
extern crate ekiden_rpc_common;
extern crate ekiden_rpc_trusted;

extern crate key_manager_api;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

mod client;

pub use client::KeyManager;
