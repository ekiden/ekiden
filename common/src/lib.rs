#![cfg_attr(feature = "sgx", no_std)]
#![cfg_attr(feature = "sgx", feature(prelude_import))]

#[cfg(feature = "sgx")]
#[cfg_attr(feature = "sgx", macro_use)]
extern crate sgx_tstd as std;

#[cfg(feature = "sgx")]
extern crate sgx_trts;

#[cfg(not(feature = "sgx"))]
extern crate rand;

extern crate protobuf;

#[cfg_attr(feature = "sgx", allow(unused))]
#[cfg_attr(feature = "sgx", prelude_import)]
#[cfg(feature = "sgx")]
use std::prelude::v1::*;

pub mod error;
pub mod random;
pub mod serializer;

#[macro_use]
pub mod hex_encoded;
