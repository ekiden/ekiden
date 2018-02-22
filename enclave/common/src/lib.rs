#![feature(use_extern_macros)]
#![cfg_attr(feature = "sgx", no_std)]
#![cfg_attr(feature = "sgx", feature(prelude_import))]

#[cfg(feature = "sgx")]
#[cfg_attr(feature = "sgx", macro_use)]
extern crate sgx_tstd as std;

#[cfg(feature = "sgx")]
extern crate sgx_trts;

extern crate base64;
extern crate byteorder;
extern crate serde_json;

extern crate ekiden_common;

#[cfg_attr(feature = "sgx", allow(unused))]
#[cfg_attr(feature = "sgx", prelude_import)]
#[cfg(feature = "sgx")]
use std::prelude::v1::*;

pub mod quote;
