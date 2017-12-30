#![feature(prelude_import)]

#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

extern crate protobuf;
use protobuf::Message;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

mod rpc;

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
            libenclave_trusted::call(request_data, request_length, response_data, response_capacity, response_length);
        }
    }
}

/// TODO: Documentation.
pub fn call(request_data: *const u8,
            request_length: usize,
            response_data: *mut u8,
            response_capacity: usize,
            response_length: *mut usize) {
    // Parse request message.
    let request = unsafe { std::slice::from_raw_parts(request_data, request_length) };
    let request: rpc::Request = protobuf::parse_from_bytes(request).expect("Failed to parse request");

    // TODO: Invoke given method.
    println!("Request method: {}", request.method);

    // Prepare response.
    let mut response = rpc::Response::new();
    response.set_code(rpc::Response_Code::SUCCESS);
    let response = response.write_to_bytes().expect("Failed to create response");

    // Copy back response.
    if response.len() > response_capacity {
        panic!("Not enough space for response.");
    } else {
        unsafe {
            for i in 0..response.len() as isize {
                std::ptr::write(response_data.offset(i), response[i as usize]);
            }

            *response_length = response.len();
        };
    }
}

// TODO: Register method.
