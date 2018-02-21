#![cfg_attr(feature = "sgx", no_std)]
#![cfg_attr(feature = "sgx", feature(prelude_import))]

#[cfg(not(feature = "sgx"))]
extern crate compute_api;
#[cfg(not(feature = "sgx"))]
extern crate grpc;
#[cfg(not(feature = "sgx"))]
extern crate rand;
#[cfg(not(feature = "sgx"))]
extern crate tls_api;
#[cfg(not(feature = "sgx"))]
extern crate tokio_core;

#[cfg(feature = "sgx")]
extern crate sgx_tstd as std;

#[cfg(feature = "sgx")]
extern crate sgx_trts;

#[cfg_attr(feature = "sgx", allow(unused))]
#[cfg_attr(feature = "sgx", prelude_import)]
#[cfg(feature = "sgx")]
use std::prelude::v1::*;

extern crate futures;
extern crate protobuf;
extern crate sodalite;

extern crate ekiden_enclave_common;
extern crate ekiden_rpc_common;

pub mod backend;
mod secure_channel;
mod client;
mod future;

#[macro_use]
mod macros;

// Re-export.
pub use client::ContractClient;
pub use ekiden_enclave_common::quote;
pub use future::{ClientFuture, FutureExtra};
