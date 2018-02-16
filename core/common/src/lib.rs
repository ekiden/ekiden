#![feature(use_extern_macros)]
#![cfg_attr(feature = "sgx", no_std)]
#![cfg_attr(feature = "sgx", feature(prelude_import))]

#[cfg(feature = "sgx")]
extern crate sgx_tstd as std;

#[cfg_attr(feature = "sgx", allow(unused))]
#[cfg_attr(feature = "sgx", prelude_import)]
#[cfg(feature = "sgx")]
use std::prelude::v1::*;

extern crate ekiden_enclave_common;
extern crate ekiden_rpc_common;

pub mod contract;

pub use ekiden_enclave_common::{hex_encoded, hex_encoded_struct, quote, random};
pub use ekiden_enclave_common::error::{self, Error, Result};

pub use ekiden_rpc_common::rpc_api;

pub mod rpc {
    pub use ekiden_rpc_common::*;
}
