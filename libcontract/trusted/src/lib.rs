#![feature(prelude_import)]
#![no_std]
#[macro_use]
extern crate sgx_tstd as std;
extern crate protobuf;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;
use protobuf::Message;

pub mod common;
pub mod generated;

/// Emits all needed code for enclave glue.
///
/// This macro should be used to create any enclave glue that is needed for
/// the Ekiden enclaves to function correctly.
///
/// A minimal enclave is as follows:
/// ```
/// #![feature(prelude_import)]
///
/// #![no_std]
///
/// #[macro_use]
/// extern crate sgx_tstd as std;
///
/// #[macro_use]
/// extern crate libenclave_trusted;
///
/// #[allow(unused)]
/// #[prelude_import]
/// use std::prelude::v1::*;
///
/// create_enclave!();
/// ```
#[macro_export]
macro_rules! create_enclave {
    () => {
        #[no_mangle]
        pub extern "C" fn rpc_call(request_data: *const u8,
                                   request_length: usize,
                                   response_data: *mut u8,
                                   response_capacity: usize,
                                   response_length: *mut usize) {
            libcontract_trusted::common::rpc::call(request_data, request_length, response_data, response_capacity, response_length);
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
