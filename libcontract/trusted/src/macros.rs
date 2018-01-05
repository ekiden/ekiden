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
    (
        metadata {
            name = $metadata_name: ident ;
            version = $metadata_version: expr ;
        }

        $(
            rpc $method_name: ident ( $request_type: ty ) -> $response_type: ty ;
        )*
    ) => {
        #[no_mangle]
        pub extern "C" fn rpc_call(request_data: *const u8,
                                   request_length: usize,
                                   response_data: *mut u8,
                                   response_capacity: usize,
                                   response_length: *mut usize) {
            let raw_response = $crate::dispatcher::RawResponse {
                data: response_data,
                capacity: response_capacity,
                length: response_length,
            };

            // Parse request.
            let request = match $crate::dispatcher::parse_request(request_data, request_length) {
                Ok(value) => value,
                _ => {
                    $crate::dispatcher::return_error(
                        libcontract_common::api::Response_Code::ERROR_BAD_REQUEST,
                        "Unable to parse request",
                        &raw_response
                    );
                    return;
                }
            };

            // Invoke given method.
            create_enclave_methods!(
                request,
                raw_response,
                // Meta methods.
                _metadata, libcontract_common::api::MetadataRequest,
                           libcontract_common::api::MetadataResponse,
                // User-defined methods.
                $( $method_name, $request_type, $response_type ),*
            );

            // If we are still here, the method could not be found.
            $crate::dispatcher::return_error(
                libcontract_common::api::Response_Code::ERROR_METHOD_NOT_FOUND,
                "Method not found",
                &raw_response
            );
        }

        // Meta method implementations.
        fn _metadata(_request: libcontract_common::api::MetadataRequest)
            -> Result<libcontract_common::api::MetadataResponse, libcontract_common::ContractError> {

            let mut response = libcontract_common::api::MetadataResponse::new();
            response.set_name(String::from(stringify!($metadata_name)));
            response.set_version(String::from($metadata_version));

            Ok(response)
        }
    };
}

/// Internal macro for creating method invocations.
#[macro_export]
macro_rules! create_enclave_methods {
    // Match when no methods are defined.
    ($request: ident, $response: ident, ) => {};

    // Match each defined method.
    (
        $request: ident, $response: ident,
        $method_name: ident, $request_type: ty, $response_type: ty
    ) => {
        if $request.method == stringify!($method_name) {
            // Parse request payload.
            let payload: $request_type = match protobuf::parse_from_bytes(&$request.get_payload()) {
                Ok(value) => value,
                _ => {
                    $crate::dispatcher::return_error(
                        libcontract_common::api::Response_Code::ERROR_BAD_REQUEST,
                        "Unable to parse request payload",
                        &$response
                    );
                    return;
                }
            };

            // Invoke method implementation.
            let response: $response_type = match $method_name(payload) {
                Ok(value) => value,
                Err(libcontract_common::ContractError { message }) => {
                    $crate::dispatcher::return_error(
                        libcontract_common::api::Response_Code::ERROR,
                        message.as_str(),
                        &$response
                    );
                    return;
                }
            };

            $crate::dispatcher::return_success(response, &$response);
            return;
        }
    };

    // Match list of defined methods.
    (
        $request: ident, $response: ident,
        $method_name: ident, $request_type: ty, $response_type: ty,
        $($x: ident, $y: ty, $z: ty),+
    ) => {
        create_enclave_methods!($request, $response, $method_name, $request_type, $response_type);
        create_enclave_methods!($request, $response, $($x, $y, $z),+);
    };
}
