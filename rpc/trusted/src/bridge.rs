use sgx_types::*;

use std;

use protobuf::{self, Message, MessageStatic};

use ekiden_enclave_common::error::{Error, Result};
use ekiden_rpc_common::api;
use ekiden_rpc_common::client::ClientEndpoint;

use super::error::DispatchError;
use super::request::Request;
use super::response::Response;
use super::secure_channel::open_request_box;
use super::untrusted;

/// List of methods that allow plain requests. All other requests must be done over
/// a secure channel.
const PLAIN_METHODS: &'static [&'static str] = &[
    api::METHOD_CONTRACT_INIT,
    api::METHOD_CONTRACT_RESTORE,
    api::METHOD_CHANNEL_INIT,
];

/// Parse an RPC request message.
pub fn parse_request(request_data: *const u8, request_length: usize) -> Vec<Request<Vec<u8>>> {
    let raw_request = unsafe { std::slice::from_raw_parts(request_data, request_length) };
    let mut enclave_request: api::EnclaveRequest = match protobuf::parse_from_bytes(raw_request) {
        Ok(enclave_request) => enclave_request,
        _ => {
            // Malformed outer request, enclave will panic.
            panic!("Malformed enclave request");
        }
    };

    let client_requests = enclave_request.take_client_request();
    let mut requests = vec![];

    for mut client_request in client_requests.into_iter() {
        if client_request.has_encrypted_request() {
            // Encrypted request.
            let plain_request = match open_request_box(&client_request.get_encrypted_request()) {
                Ok(plain_request) => plain_request,
                _ => Request::error(DispatchError::new(
                    api::PlainClientResponse_Code::ERROR_SECURE_CHANNEL,
                    "Unable to open secure channel request",
                )),
            };

            requests.push(plain_request);
        } else {
            // Plain request.
            let mut plain_request = client_request.take_plain_request();
            let plain_request = match PLAIN_METHODS
                .iter()
                .find(|&method| method == &plain_request.get_method())
            {
                Some(_) => Request::new(
                    plain_request.take_payload(),
                    plain_request.take_method(),
                    None,
                    None,
                ),
                None => {
                    // Method requires a secure channel.
                    Request::error(DispatchError::new(
                        api::PlainClientResponse_Code::ERROR_METHOD_SECURE,
                        "Method call must be made over a secure channel",
                    ))
                }
            };

            requests.push(plain_request);
        }
    }

    requests
}

/// Serialize and return an RPC response.
pub fn return_response(
    responses: Vec<Response>,
    response_data: *mut u8,
    response_capacity: usize,
    response_length: *mut usize,
) {
    let mut enclave_response = api::EnclaveResponse::new();

    // Add all responses.
    {
        let client_responses = enclave_response.mut_client_response();
        for mut response in responses {
            client_responses.push(response.take_message());
        }
    }

    // TODO: Return null response instead?
    let enclave_response_bytes = enclave_response
        .write_to_bytes()
        .expect("Failed to serialize response");

    // Copy back response.
    if enclave_response_bytes.len() > response_capacity {
        // TODO: Return null response instead?
        panic!("Not enough space for response.");
    } else {
        unsafe {
            for i in 0..enclave_response_bytes.len() as isize {
                std::ptr::write(response_data.offset(i), enclave_response_bytes[i as usize]);
            }
            *response_length = enclave_response_bytes.len();
        };
    }
}

/// Perform an untrusted RPC call against a given (untrusted) endpoint.
///
/// How the actual RPC call is implemented depends on the handler implemented
/// in the untrusted part.
pub fn untrusted_call_endpoint<Rq, Rs>(endpoint: &ClientEndpoint, request: Rq) -> Result<Rs>
where
    Rq: Message,
    Rs: Message + MessageStatic,
{
    Ok(protobuf::parse_from_bytes(&untrusted_call_endpoint_raw(
        &endpoint,
        request.write_to_bytes()?,
    )?)?)
}

/// Perform a raw RPC call against a given (untrusted) endpoint.
///
/// How the actual RPC call is implemented depends on the handler implemented
/// in the untrusted part.
pub fn untrusted_call_endpoint_raw(
    endpoint: &ClientEndpoint,
    mut request: Vec<u8>,
) -> Result<Vec<u8>> {
    // Maximum size of serialized response is 64K.
    let mut response: Vec<u8> = Vec::with_capacity(64 * 1024);

    // Ensure that request is actually allocated as the length of the actual request
    // may be zero and in that case the OCALL will fail with SGX_ERROR_INVALID_PARAMETER.
    request.reserve(1);

    let mut response_length = 0;
    let status = unsafe {
        untrusted::untrusted_rpc_call(
            endpoint.as_u16(),
            request.as_ptr() as *const u8,
            request.len(),
            response.as_mut_ptr() as *mut u8,
            response.capacity(),
            &mut response_length,
        )
    };

    match status {
        sgx_status_t::SGX_SUCCESS => {}
        status => {
            return Err(Error::new(&format!(
                "Enclave RPC OCALL failed: {:?}",
                status
            )));
        }
    }

    unsafe {
        response.set_len(response_length);
    }

    Ok(response)
}
