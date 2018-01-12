extern crate futures;
extern crate futures_cpupool;
extern crate protobuf;
extern crate grpc;
extern crate tls_api;
extern crate byteorder;
extern crate rand;
extern crate sodalite;

extern crate libcontract_common;

mod generated;
mod client;
mod errors;
mod ias;

#[macro_use]
mod macros;

// Re-export.
pub use client::{ContractClient, ContractStatus};
pub use errors::Error;
