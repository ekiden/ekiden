#![cfg_attr(feature = "sgx", no_std)]
#![cfg_attr(feature = "sgx", feature(prelude_import))]

extern crate protobuf;

#[cfg(feature = "sgx")]
extern crate sgx_tstd as std;

#[cfg_attr(feature = "sgx", allow(unused))]
#[cfg_attr(feature = "sgx", prelude_import)]
#[cfg(feature = "sgx")]
use std::prelude::v1::*;

pub mod address;
pub mod contract;
pub mod contract_error;

mod generated;

#[macro_use]
mod macros;

pub use address::Address;
pub use contract::*;
pub use contract_error::ContractError;

pub mod api {
    pub use generated::enclave_rpc::{Request, Response, Response_Code, Error, MetadataRequest, MetadataResponse};
}
