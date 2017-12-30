
#![no_std]
#![feature(prelude_import)]
#[prelude_import]
use std::prelude::v1::*;

#[macro_use]
extern crate sgx_tstd as std;
extern crate protobuf;

pub mod address;
pub mod contract;
pub mod contract_error;
pub mod generated;
pub mod rpc;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
