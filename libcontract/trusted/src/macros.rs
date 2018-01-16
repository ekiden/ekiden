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

            let mut raw_response = $crate::dispatcher::RawResponse {
                data: response_data,
                capacity: response_capacity,
                length: response_length,
                public_key: vec![],
            };

            // Parse request.
            let request = match $crate::dispatcher::parse_request(
                request_data,
                request_length,
                &mut raw_response
            ) {
                Ok(value) => value,
                _ => {
                    // A suitable response has already been generated.
                    return;
                }
            };

            // Special handling methods.
            match request.get_method() {
                // Special handling for channel close as it requires to know the caller
                // channel identity and to generate the response before closing the channel.
                "_channel_close" => {
                    // Prepare response before closing the channel.
                    let response = api::ChannelCloseResponse::new();
                    $crate::dispatcher::return_success(response, &raw_response);

                    match $crate::secure_channel::channel_close(&raw_response.public_key) {
                        Ok(_) => {},
                        _ => {
                            // Errors are ignored.
                        }
                    };
                    return;
                },
                _ => {},
            }

            // Invoke given method.
            use libcontract_common::api;

            create_enclave_methods!(
                request,
                raw_response,
                // Meta methods.
                _metadata, api::MetadataRequest, api::MetadataResponse,
                _contract_init, api::ContractInitRequest, api::ContractInitResponse,
                _contract_restore, api::ContractRestoreRequest, api::ContractRestoreResponse,
                _channel_init, api::ChannelInitRequest, api::ChannelInitResponse,
                // User-defined methods.
                $( $method_name, $request_type, $response_type ),*
            );

            // If we are still here, the method could not be found.
            $crate::dispatcher::return_error(
                libcontract_common::api::PlainResponse_Code::ERROR_METHOD_NOT_FOUND,
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

        fn _contract_init(request: libcontract_common::api::ContractInitRequest)
            -> Result<libcontract_common::api::ContractInitResponse, libcontract_common::ContractError> {

            $crate::secure_channel::contract_init(request)
        }

        fn _contract_restore(request: libcontract_common::api::ContractRestoreRequest)
            -> Result<libcontract_common::api::ContractRestoreResponse, libcontract_common::ContractError> {

            $crate::secure_channel::contract_restore(request)
        }

        fn _channel_init(request: libcontract_common::api::ChannelInitRequest)
            -> Result<libcontract_common::api::ChannelInitResponse, libcontract_common::ContractError> {

            $crate::secure_channel::channel_init(request)
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
                        libcontract_common::api::PlainResponse_Code::ERROR_BAD_REQUEST,
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
                        libcontract_common::api::PlainResponse_Code::ERROR,
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
