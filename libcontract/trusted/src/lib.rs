#![feature(prelude_import)]
#![no_std]

#[macro_use]
extern crate sgx_tstd as std;
extern crate protobuf;

extern crate libcontract_common;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

pub mod dispatcher;
pub mod errors;

#[macro_use]
mod macros;
