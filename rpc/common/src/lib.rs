#![cfg_attr(feature = "sgx", no_std)]
#![cfg_attr(feature = "sgx", feature(prelude_import))]

extern crate byteorder;
extern crate protobuf;
extern crate sodalite;

#[cfg(feature = "sgx")]
#[cfg_attr(feature = "sgx", macro_use)]
extern crate sgx_tstd as std;

#[cfg(feature = "sgx")]
extern crate sgx_trts;

extern crate ekiden_enclave_common;

#[cfg_attr(feature = "sgx", allow(unused))]
#[cfg_attr(feature = "sgx", prelude_import)]
#[cfg(feature = "sgx")]
use std::prelude::v1::*;

pub mod reflection;
pub mod serializer;
pub mod secure_channel;
pub mod client;

mod generated;

#[macro_use]
mod macros;

mod protocol;

pub mod api {
    pub use generated::enclave_rpc::*;
    pub use protocol::*;

    pub mod services {
        pub use generated::enclave_services::*;
    }
}
