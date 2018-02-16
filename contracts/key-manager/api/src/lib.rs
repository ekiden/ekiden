#![cfg_attr(feature = "sgx", no_std)]
#![cfg_attr(feature = "sgx", feature(prelude_import))]

extern crate protobuf;

#[cfg(feature = "sgx")]
extern crate sgx_tstd as std;

#[macro_use]
extern crate ekiden_core_common;

#[cfg_attr(feature = "sgx", allow(unused))]
#[cfg_attr(feature = "sgx", prelude_import)]
#[cfg(feature = "sgx")]
use std::prelude::v1::*;

#[macro_use]
mod api;
mod generated;

pub use generated::api::*;
