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
            use protobuf::Message;

            use libcontract_common::{api, ContractError};

            use $crate::dispatcher::{parse_request, return_response, Request, Response};
            use $crate::secure_channel::{channel_init, contract_init, contract_restore};
            #[allow(unused)]
            use $crate::state_crypto::{decrypt_state, encrypt_state};

            use $crate::state_diffs;

            use super::*;

            #[no_mangle]
            pub extern "C" fn rpc_call(request_data: *const u8,
                                       request_length: usize,
                                       response_data: *mut u8,
                                       response_capacity: usize,
                                       response_length: *mut usize) {

                // Parse request.
                let (encrypted_state, requests) = match parse_request(
                    request_data,
                    request_length,
                ) {
                    Ok(value) => value,
                    _ => unreachable!()
                };

                // Decrypt starting state.
                let mut state: Option<$metadata_state_type> = match encrypted_state {
                    Some(encrypted_state) =>
                        match decrypt_state(&encrypted_state) {
                            Ok(value) =>
                                match protobuf::parse_from_bytes(&value) {
                                    Ok(value) => Some(value),
                                    Err(_) => None,
                                },
                            Err(_) => None,
                        },
                    None => None,
                };

                // Process requests.
                let mut have_state_updates = false;
                let mut responses = vec![];
                for request in requests {
                    let response = if let &Some(ref error) = request.get_error() {
                        // Error occurred during request processing, forward it.
                        Response::error(&request, error.code, &error.message)
                    } else {
                        let mut response = handle_request(&state, &request);
                        match response.take_state() {
                            Some(new_state) => {
                                // Response updates state.
                                state = Some(new_state);
                                have_state_updates = true;
                            },
                            None => {}
                        }

                        response
                    };

                    responses.push(response);
                }

                // Encrypt state if there were any updates.
                let encrypted_state = if have_state_updates {
                    match state {
                        Some(state) => {
                            let state_bytes = state.write_to_bytes().expect("Failed to serialize state");
                            Some(encrypt_state(state_bytes).expect("Failed to encrypt state"))
                        }
                        None => None,
                    }
                } else {
                    None
                };

                // Generate response.
                return_response(
                    encrypted_state,
                    responses,
                    response_data,
                    response_capacity,
                    response_length,
                );
            }

            #[allow(unused)]
            fn handle_request(
                state: &Option<$metadata_state_type>,
                request: &Request<api::PlainClientRequest>
            ) -> Response<$metadata_state_type> {

                // Special handling methods.
                match request.get_method() {
                    // Special handling for channel close as it requires to know the caller
                    // channel identity and to generate the response before closing the channel.
                    "_channel_close" => {
                        // Prepare response before closing the channel.
                        let response = Response::success(
                            &request,
                            api::ChannelCloseResponse::new()
                        );

                        if let &Some(ref public_key) = request.get_client_public_key() {
                            match $crate::secure_channel::channel_close(&public_key) {
                                Ok(_) => {},
                                _ => {
                                    // Errors are ignored.
                                }
                            }
                        }

                        return response;
                    },
                    _ => {},
                }

                // Meta methods. Keep these names in sync with libcontract/common/src/protocol.rs.
                create_enclave_method!(
                    state,
                    request,
                    $metadata_state_type,
                    _metadata(api::MetadataRequest) -> api::MetadataResponse
                );
                create_enclave_method!(
                    state,
                    request,
                    $metadata_state_type,
                    _contract_init(api::ContractInitRequest) -> api::ContractInitResponse
                );
                create_enclave_method!(
                    state,
                    request,
                    $metadata_state_type,
                    _contract_restore(api::ContractRestoreRequest) -> api::ContractRestoreResponse
                );
                create_enclave_method!(
                    state,
                    request,
                    $metadata_state_type,
                    _channel_init(api::ChannelInitRequest) -> api::ChannelInitResponse
                );
                create_enclave_method!(
                    state,
                    request,
                    $metadata_state_type,
                    _state_diff(api::StateDiffRequest) -> api::StateDiffResponse
                );
                create_enclave_method!(
                    state,
                    request,
                    $metadata_state_type,
                    _state_apply(api::StateApplyRequest) -> api::StateApplyResponse
                );

                // User-defined methods.
                $(
                    create_enclave_method!(
                        state,
                        request,
                        $metadata_state_type,
                        $method_name $method_in -> $method_out
                    );
                )*

                // If we are still here, the method could not be found.
                return Response::error(
                    &request,
                    api::PlainClientResponse_Code::ERROR_METHOD_NOT_FOUND,
                    "Method not found"
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

            fn _state_diff(request: &api::StateDiffRequest) ->
                Result<api::StateDiffResponse, ContractError>
            {
                state_diffs::diff(request)
            }

            fn _state_apply(request: &api::StateApplyRequest) ->
                Result<api::StateApplyResponse, ContractError>
            {
                state_diffs::apply(request)
            }
        }

        // Re-export rpc_call.
        pub use self::enclave_rpc::rpc_call;
    };
}

#[macro_export]
macro_rules! parse_enclave_method_state {
    ( $state: ident, $request: ident, $state_type: ty ) => {{
        // Ensure state is passed.
        if $state.is_none() {
            return Response::error(
                &$request,
                api::PlainClientResponse_Code::ERROR_BAD_REQUEST,
                "Request must come with state",
            );
        }

        $state.as_ref().unwrap()
    }}
}

#[macro_export]
macro_rules! parse_enclave_method_request {
    ( $request: ident, $request_type: ty ) => {{
        let payload: Request<$request_type> = match protobuf::parse_from_bytes(
            &$request.get_payload()
        ) {
            Ok(value) => $request.copy_metadata_to(value),
            _ => {
                return Response::error(
                    &$request,
                    api::PlainClientResponse_Code::ERROR_BAD_REQUEST,
                    "Unable to parse request payload",
                );
            }
        };

        payload
    }}
}

#[macro_export]
macro_rules! handle_enclave_method_invocation {
    ( $request: ident, $invocation: expr ) => {{
        match $invocation {
            Ok(value) => {
                value
            }
            Err(ContractError { message }) => {
                return Response::error(
                    &$request,
                    api::PlainClientResponse_Code::ERROR,
                    message.as_str(),
                );
            }
        }
    }}
}

/// Internal macro for creating method invocations.
#[macro_export]
macro_rules! create_enclave_method {
    // State in, state out. E.g., transactions
    (
        $state: ident, $request: ident, $state_type: ty,
        $method_name: ident ( state , $request_type: ty ) -> ( state , $response_type: ty )
    ) => {
        if $request.method == stringify!($method_name) {
            let state = parse_enclave_method_state!($state, $request, $state_type);
            let payload = parse_enclave_method_request!($request, $request_type);

            // Invoke method implementation.
            let (new_state, response): ($state_type, $response_type) =
                handle_enclave_method_invocation!($request, $method_name(&state, &payload));

            return Response::success(&$request, response).with_state(new_state);
        }
    };
    // No state in, state out. E.g., initializers
    (
        $state: ident, $request: ident, $state_type: ty,
        $method_name: ident ( $request_type: ty ) -> ( state , $response_type: ty )
    ) => {
        if $request.method == stringify!($method_name) {
            let payload = parse_enclave_method_request!($request, $request_type);

            // Invoke method implementation.
            let (new_state, response): ($state_type, $response_type) =
                handle_enclave_method_invocation!($request, $method_name(&payload));

            return Response::success(&$request, response).with_state(new_state);
        }
    };
    // State in, no state out. E.g., reads
    (
        $state: ident, $request: ident, $state_type: ty,
        $method_name: ident ( state , $request_type: ty ) -> $response_type: ty
    ) => {
        if $request.method == stringify!($method_name) {
            let state = parse_enclave_method_state!($state, $request, $state_type);
            let payload = parse_enclave_method_request!($request, $request_type);

            // Invoke method implementation.
            let response: $response_type =
                handle_enclave_method_invocation!($request, $method_name(&state, &payload));

            return Response::success(&$request, response);
        }
    };
    // No state in, no state out. E.g., _metadata
    (
        $state: ident, $request: ident, $state_type: ty,
        $method_name: ident ( $request_type: ty ) -> $response_type: ty
    ) => {
        if $request.method == stringify!($method_name) {
            let payload = parse_enclave_method_request!($request, $request_type);

            // Invoke method implementation.
            let response: $response_type =
                handle_enclave_method_invocation!($request, $method_name(&payload));

            return Response::success(&$request, response);
        }
    };
}
