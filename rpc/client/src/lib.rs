#[cfg(not(target_env = "sgx"))]
extern crate grpc;
#[cfg(not(target_env = "sgx"))]
extern crate rand;
#[cfg(not(target_env = "sgx"))]
extern crate tls_api;
#[cfg(not(target_env = "sgx"))]
extern crate tokio_core;

extern crate futures;
extern crate protobuf;
extern crate sodalite;

extern crate ekiden_common;
#[cfg(not(target_env = "sgx"))]
extern crate ekiden_compute_api;
extern crate ekiden_enclave_common;
extern crate ekiden_rpc_common;

pub mod backend;
mod secure_channel;
mod client;
mod future;

#[macro_use]
mod macros;

// Re-export.
pub use client::ContractClient;
pub use ekiden_enclave_common::quote;
pub use future::{ClientFuture, FutureExtra};
