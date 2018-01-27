#![feature(prelude_import)]
#![feature(use_extern_macros)]
#![no_std]

extern crate sgx_trts;
extern crate sgx_tse;
extern crate sgx_tseal;
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_types;

#[macro_use]
extern crate lazy_static;
extern crate protobuf;
extern crate sodalite;

extern crate compute_client;
extern crate key_manager_api;
extern crate libcontract_common;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

pub mod quote;
pub mod secure_channel;
pub mod dispatcher;
pub mod errors;
pub mod state_crypto;
pub mod state_diffs;
pub mod key_manager;

#[macro_use]
mod macros;

mod untrusted;
mod client;
