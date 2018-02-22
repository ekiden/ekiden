#![feature(use_extern_macros)]

extern crate sgx_trts;
extern crate sgx_tse;
extern crate sgx_tseal;
extern crate sgx_types;

extern crate bsdiff;
extern crate bzip2;
#[macro_use]
extern crate lazy_static;
extern crate protobuf;
extern crate sodalite;

extern crate ekiden_common;
extern crate key_manager_client;

mod generated;

mod crypto;
mod diffs;
pub mod ecalls;

pub mod db;
pub use db::Db;
