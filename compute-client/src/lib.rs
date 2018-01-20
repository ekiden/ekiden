#![cfg_attr(feature = "sgx", no_std)]
#![cfg_attr(feature = "sgx", feature(prelude_import))]

#[cfg(not(feature = "sgx"))]
extern crate grpc;
#[cfg(not(feature = "sgx"))]
extern crate rand;
#[cfg(not(feature = "sgx"))]
extern crate tls_api;

#[cfg(feature = "sgx")]
#[cfg_attr(feature = "sgx", macro_use)]
extern crate sgx_tstd as std;

#[cfg(feature = "sgx")]
extern crate sgx_trts;

#[cfg_attr(feature = "sgx", allow(unused))]
#[cfg_attr(feature = "sgx", prelude_import)]
#[cfg(feature = "sgx")]
use std::prelude::v1::*;

extern crate protobuf;
extern crate sodalite;

extern crate libcontract_common;

#[cfg(not(feature = "sgx"))]
mod generated;

pub mod backend;
mod client;
mod errors;

#[macro_use]
mod macros;

// Re-export.
pub use client::ContractClient;
pub use errors::Error;
