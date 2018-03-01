#![feature(use_extern_macros)]
#![feature(core_intrinsics)]

extern crate sgx_tse;
extern crate sgx_tseal;
extern crate sgx_types;

extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate protobuf;
extern crate sodalite;

extern crate ekiden_common;
extern crate ekiden_enclave_common;
extern crate ekiden_rpc_client;
extern crate ekiden_rpc_common;

pub mod bridge;
pub mod dispatcher;
pub mod error;
pub mod request;
pub mod response;

pub mod quote;
pub mod secure_channel;

#[macro_use]
mod macros;

mod untrusted;
pub mod client;
