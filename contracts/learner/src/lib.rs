#![cfg_attr(feature = "sgx", no_std)]
#![cfg_attr(feature = "sgx", feature(prelude_import))]

#[cfg(feature = "sgx")]
#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

#[cfg(feature = "sgx")]
#[macro_use]
extern crate sgx_tstd as std;

extern crate protobuf;
extern crate rusty_machine;
extern crate serde;
extern crate serde_cbor;

extern crate libcontract_common;

pub extern crate learner_api as api;

mod contract;
#[macro_use]
mod macros;
pub mod utils;

pub use rusty_machine::prelude::*;

pub use contract::Learner;
pub use utils::*;
