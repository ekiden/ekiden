#![feature(use_extern_macros)]

extern crate ekiden_db_untrusted;
extern crate ekiden_enclave_untrusted;
extern crate ekiden_rpc_untrusted;

pub use ekiden_db_untrusted::EnclaveDb;
pub use ekiden_enclave_untrusted::Enclave;
pub use ekiden_rpc_untrusted::EnclaveRpc;

#[macro_use]
pub mod rpc;
