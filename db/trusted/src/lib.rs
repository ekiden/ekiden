#![feature(use_extern_macros)]

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
