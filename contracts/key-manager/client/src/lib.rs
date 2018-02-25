#![feature(use_extern_macros)]

#[macro_use]
extern crate lazy_static;

extern crate ekiden_common;
extern crate ekiden_enclave_common;
extern crate ekiden_rpc_client;
extern crate ekiden_rpc_common;
extern crate ekiden_rpc_trusted;

extern crate key_manager_api;

mod client;

pub use client::KeyManager;
