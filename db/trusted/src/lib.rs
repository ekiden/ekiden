#![feature(prelude_import)]
#![feature(use_extern_macros)]
#![no_std]

extern crate sgx_trts;
extern crate sgx_tse;
extern crate sgx_tseal;
#[macro_use]
extern crate sgx_tstd as std;
extern crate sgx_types;

extern crate bsdiff;
extern crate bzip2;
#[macro_use]
extern crate lazy_static;
extern crate protobuf;
extern crate sodalite;

extern crate ekiden_common;
extern crate key_manager_client;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

mod generated;

mod crypto;
mod diffs;
pub mod ecalls;

pub mod db;
pub use db::Db;
