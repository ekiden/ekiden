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
/// extern crate libcontract_common;
///
/// extern crate protobuf;
///
/// #[allow(unused)]
/// #[prelude_import]
/// use std::prelude::v1::*;
///
/// create_enclave_api!();
/// ```
#[macro_export]
macro_rules! create_enclave {
    (
        metadata {
            name = $metadata_name: ident ;
            version = $metadata_version: expr ;
            state_type = $metadata_state_type: ty ;
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

            // Parse starting state.
            let state: Option<$metadata_state_type> = {
                let raw_state = request.get_state();
                if raw_state.len() == 0 {
                    None
                } else {
                    // TODO: decrypt state
                    match protobuf::parse_from_bytes(raw_state) {
                        Ok(value) => Some(value),
                        _ => {
                            $crate::dispatcher::return_error(
                                libcontract_common::api::Response_Code::ERROR_BAD_REQUEST,
                                "Unable to parse request state",
                                &raw_response
                            );
                            return;
                        }
                    }
                }
            };

            // Invoke given method.

            // Meta methods.
            create_enclave_method!(
                state,
                request,
                raw_response,
                $metadata_state_type,
                _metadata, libcontract_common::api::MetadataRequest,
                libcontract_common::api::MetadataResponse
            );

            // User-defined methods.
            $(
                create_enclave_method!(
                    state,
                    request,
                    raw_response,
                    $metadata_state_type,
                    $method_name, $request_type, $response_type
                );
            )*

            // If we are still here, the method could not be found.
            $crate::dispatcher::return_error(
                libcontract_common::api::Response_Code::ERROR_METHOD_NOT_FOUND,
                "Method not found",
                &raw_response
            );
        }

        // Meta method implementations.
        fn _metadata(state: Option<$metadata_state_type>, _request: libcontract_common::api::MetadataRequest)
            -> Result<(Option<$metadata_state_type>, libcontract_common::api::MetadataResponse), libcontract_common::ContractError> {
            assert!(state.is_none());

            let mut response = libcontract_common::api::MetadataResponse::new();
            response.set_name(String::from(stringify!($metadata_name)));
            response.set_version(String::from($metadata_version));

            Ok((None, response))
        }
    };
}

/// Internal macro for creating method invocations.
#[macro_export]
macro_rules! create_enclave_method {
    (
        $state: ident, $request: ident, $response: ident, $state_type: ty,
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
            let (new_state, response): (Option<$state_type>, $response_type) = match $method_name($state, payload) {
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

            $crate::dispatcher::return_success(new_state, response, &$response);
            return;
        }
    };
}
