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
            client_attestation_required = $client_attestation_required: expr ;
        }

        $(
            rpc $method_name: ident $method_in: tt -> $method_out: tt ;
        )*
    ) => {
        mod enclave_rpc {
            use protobuf;

            use libcontract_common::{api, ContractError};

            use $crate::dispatcher::{parse_request, return_error, return_success, RawResponse,
                                     Request};
            use $crate::secure_channel::{channel_attest_client, channel_init, contract_init,
                                         contract_restore};
            #[allow(unused)]
            use $crate::state_crypto::decrypt_state;

            use super::*;

            #[no_mangle]
            pub extern "C" fn rpc_call(request_data: *const u8,
                                       request_length: usize,
                                       response_data: *mut u8,
                                       response_capacity: usize,
                                       response_length: *mut usize) {

                let mut raw_response = RawResponse {
                    data: response_data,
                    capacity: response_capacity,
                    length: response_length,
                    public_key: vec![],
                };

                // Parse request.
                #[allow(unused)]
                let (encrypted_state, request) = match parse_request(
                    request_data,
                    request_length,
                    &mut raw_response
                ) {
                    Ok(value) => value,
                    _ => {
                        // Parsing failed, and a suitable error response has been sent.
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
                        return_success(
                            None::<$metadata_state_type>, response, &raw_response
                        );

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

                // Meta methods.
                create_enclave_method!(
                    encrypted_state,
                    request,
                    raw_response,
                    $metadata_state_type,
                    _metadata(api::MetadataRequest) -> api::MetadataResponse
                );
                create_enclave_method!(
                    encrypted_state,
                    request,
                    raw_response,
                    $metadata_state_type,
                    _contract_init(api::ContractInitRequest) -> api::ContractInitResponse
                );
                create_enclave_method!(
                    encrypted_state,
                    request,
                    raw_response,
                    $metadata_state_type,
                    _contract_restore(api::ContractRestoreRequest) -> api::ContractRestoreResponse
                );
                create_enclave_method!(
                    encrypted_state,
                    request,
                    raw_response,
                    $metadata_state_type,
                    _channel_init(api::ChannelInitRequest) -> api::ChannelInitResponse
                );
                create_enclave_method!(
                    encrypted_state,
                    request,
                    raw_response,
                    $metadata_state_type,
                    _channel_attest_client(
                        api::ChannelAttestClientRequest
                    ) -> api::ChannelAttestClientResponse
                );

                // User-defined methods.
                $(
                    create_enclave_method!(
                        encrypted_state,
                        request,
                        raw_response,
                        $metadata_state_type,
                        $method_name $method_in -> $method_out
                    );
                )*

                // If we are still here, the method could not be found.
                return_error(
                    api::PlainClientResponse_Code::ERROR_METHOD_NOT_FOUND,
                    "Method not found",
                    &raw_response
                );
            }

            // Meta method implementations.
            fn _metadata(_request: &api::MetadataRequest) ->
                Result<api::MetadataResponse, ContractError>
            {
                let mut response = api::MetadataResponse::new();
                response.set_name(String::from(stringify!($metadata_name)));
                response.set_version(String::from($metadata_version));

                Ok(response)
            }

            fn _contract_init(request: &api::ContractInitRequest) ->
                Result<api::ContractInitResponse, ContractError>
            {

                contract_init(request)
            }

            fn _contract_restore(request: &api::ContractRestoreRequest) ->
                Result<
                    api::ContractRestoreResponse,
                    ContractError
                >
            {
                contract_restore(request)
            }

            fn _channel_init(request: &api::ChannelInitRequest) ->
                Result<api::ChannelInitResponse, ContractError>
            {
                channel_init(request, $client_attestation_required)
            }

            fn _channel_attest_client(request: &Request<api::ChannelAttestClientRequest>) ->
                Result<api::ChannelAttestClientResponse, ContractError>
            {
                channel_attest_client(request)
            }
        }

        // Re-export rpc_call.
        pub use self::enclave_rpc::rpc_call;
    };
}

#[macro_export]
macro_rules! parse_enclave_method_state {
    ( $state: ident, $response: ident, $state_type: ty ) => {{
        // Decrypt starting state.
        let state: $state_type = match $state {
            Some(encrypted_state) =>
                match decrypt_state(&encrypted_state) {
                    Ok(value) => value,
                    _ => {
                        return_error(
                            api::PlainClientResponse_Code::ERROR_BAD_REQUEST,
                            "Unable to parse request state",
                            &$response
                        );
                        return;
                    }
                },
            None => {
                return_error(
                    api::PlainClientResponse_Code::ERROR_BAD_REQUEST,
                    "Request must come with state",
                    &$response
                );
                return;
            },
        };

        state
    }}
}

#[macro_export]
macro_rules! parse_enclave_method_request {
    ( $request: ident, $response: ident, $request_type: ty ) => {{
        let payload: Request<$request_type> = match protobuf::parse_from_bytes(
            &$request.get_payload()
        ) {
            Ok(value) => $request.copy_metadata_to(value),
            _ => {
                return_error(
                    api::PlainClientResponse_Code::ERROR_BAD_REQUEST,
                    "Unable to parse request payload",
                    &$response
                );
                return;
            }
        };

        payload
    }}
}

#[macro_export]
macro_rules! handle_enclave_method_invocation {
    ( $response: ident, $invocation: expr ) => {{
        match $invocation {
            Ok(value) => value,
            Err(ContractError { message }) => {
                return_error(
                    api::PlainClientResponse_Code::ERROR,
                    message.as_str(),
                    &$response
                );
                return;
            }
        }
    }}
}

/// Internal macro for creating method invocations.
#[macro_export]
macro_rules! create_enclave_method {
    // State in, state out. E.g., transactions
    (
        $state: ident, $request: ident, $response: ident, $state_type: ty,
        $method_name: ident ( state , $request_type: ty ) -> ( state , $response_type: ty )
    ) => {
        if $request.method == stringify!($method_name) {
            let state = parse_enclave_method_state!($state, $response, $state_type);
            let payload = parse_enclave_method_request!($request, $response, $request_type);

            // Invoke method implementation.
            let (new_state, response): ($state_type, $response_type) =
                handle_enclave_method_invocation!($response, $method_name(&state, &payload));

            return_success(Some(new_state), response, &$response);
            return;
        }
    };
    // No state in, state out. E.g., initializers
    (
        $state: ident, $request: ident, $response: ident, $state_type: ty,
        $method_name: ident ( $request_type: ty ) -> ( state , $response_type: ty )
    ) => {
        if $request.method == stringify!($method_name) {
            let payload = parse_enclave_method_request!($request, $response, $request_type);

            // Invoke method implementation.
            let (new_state, response): ($state_type, $response_type) =
                handle_enclave_method_invocation!($response, $method_name(&payload));

            return_success(Some(new_state), response, &$response);
            return;
        }
    };
    // State in, no state out. E.g., reads
    (
        $state: ident, $request: ident, $response: ident, $state_type: ty,
        $method_name: ident ( state , $request_type: ty ) -> $response_type: ty
    ) => {
        if $request.method == stringify!($method_name) {
            let state = parse_enclave_method_state!($state, $response, $state_type);
            let payload = parse_enclave_method_request!($request, $response, $request_type);

            // Invoke method implementation.
            let response: $response_type =
                handle_enclave_method_invocation!($response, $method_name(&state, &payload));

            return_success(None::<$state_type>, response, &$response);
            return;
        }
    };
    // No state in, no state out. E.g., _metadata
    (
        $state: ident, $request: ident, $response: ident, $state_type: ty,
        $method_name: ident ( $request_type: ty ) -> $response_type: ty
    ) => {
        if $request.method == stringify!($method_name) {
            let payload = parse_enclave_method_request!($request, $response, $request_type);

            // Invoke method implementation.
            let response: $response_type =
                handle_enclave_method_invocation!($response, $method_name(&payload));

            return_success(None::<$state_type>, response, &$response);
            return;
        }
    };
}
