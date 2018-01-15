use std;

use protobuf;
use protobuf::Message;

use libcontract_common::api;

use super::secure_channel;

/// Raw data needed to generate the response.
pub struct RawResponse {
    /// Response output buffer.
    pub data: *mut u8,
    /// Response buffer capacity.
    pub capacity: usize,
    /// Response output length.
    pub length: *mut usize,
    /// Client public key (for encrypted requests).
    pub public_key: Vec<u8>,
}

/// List of methods that allow plain requests. All other requests must be done over
/// a secure channel.
const PLAIN_METHODS: &'static [&'static str] = &[
    "_metadata",
    "_contract_init",
    "_contract_restore",
    "_channel_init",
];

/// Parse an RPC request message.
pub fn parse_request(request_data: *const u8,
                     request_length: usize,
                     raw_response: &mut RawResponse) -> Result<api::PlainRequest, ()> {

    let request = unsafe { std::slice::from_raw_parts(request_data, request_length) };
    let mut request: api::Request = match protobuf::parse_from_bytes(request) {
        Ok(request) => request,
        _ => {
            return_error(
                api::PlainResponse_Code::ERROR_BAD_REQUEST,
                "Unable to parse request",
                &raw_response
            );
            return Err(());
        }
    };

    if request.has_encrypted_request() {
        // Encrypted request.
        raw_response.public_key = request.get_encrypted_request().get_public_key().to_vec();
        match secure_channel::open_request_box(&request.get_encrypted_request()) {
            Ok(plain_request) => Ok(plain_request),
            _ => {
                return_error(
                    api::PlainResponse_Code::ERROR_SECURE_CHANNEL,
                    "Unable to open secure channel request",
                    &raw_response
                );
                Err(())
            }
        }
    } else {
        // Plain request.
        let plain_request = request.take_plain_request();
        match PLAIN_METHODS.iter().find(|&method| method == &plain_request.get_method()) {
            Some(_) => {},
            None => {
                // Method requires a secure channel.
                return_error(
                    api::PlainResponse_Code::ERROR_METHOD_SECURE,
                    "Method call must be made over a secure channel",
                    &raw_response
                );
                return Err(());
            }
        };

        Ok(plain_request)
    }
}

/// Serialize and return an RPC response.
pub fn return_response(plain_response: api::PlainResponse,
                       raw_response: &RawResponse) {

    let mut response = api::Response::new();

    if raw_response.public_key.is_empty() {
        // Plain response.
        response.set_plain_response(plain_response);
    } else {
        // Encrypted response.
        match secure_channel::create_response_box(
            &raw_response.public_key,
            &plain_response
        ) {
            Ok(response_box) => response.set_encrypted_response(response_box),
            _ => {
                // Failed to create a cryptographic box for the response. This could
                // be due to the session being incorrect or due to other issues. In
                // this case, we should generate a plain error message.
                response.set_plain_response(generate_error(
                    api::PlainResponse_Code::ERROR_SECURE_CHANNEL,
                    "Failed to generate secure channel response"
                ));
            }
        };
    }

    // TODO: Return null response instead?
    let response = response.write_to_bytes().expect("Failed to serialize response");

    // Copy back response.
    if response.len() > raw_response.capacity {
        // TODO: Return null response instead?
        panic!("Not enough space for response.");
    } else {
        unsafe {
            for i in 0..response.len() as isize {
                std::ptr::write(raw_response.data.offset(i), response[i as usize]);
            }
            *raw_response.length = response.len();
        };
    }
}

/// Generate error response.
pub fn generate_error(error: api::PlainResponse_Code,
                      message: &str) -> api::PlainResponse {

    // Prepare response.
    let mut response = api::PlainResponse::new();
    response.set_code(error);

    let mut error = api::Error::new();
    error.set_message(message.to_string());

    let payload = error.write_to_bytes().expect("Failed to serialize error");
    response.set_payload(payload);

    response
}

/// Serialize and return an RPC success response.
pub fn return_success<M: Message>(payload: M,
                                  raw_response: &RawResponse) {

    // Prepare response.
    let mut response = api::PlainResponse::new();
    response.set_code(api::PlainResponse_Code::SUCCESS);

    let payload = payload.write_to_bytes().expect("Failed to serialize payload");
    response.set_payload(payload);

    return_response(
        response,
        raw_response
    );
}

/// Serialize and return an RPC error response.
pub fn return_error(error: api::PlainResponse_Code,
                    message: &str,
                    raw_response: &RawResponse) {

    return_response(
        generate_error(error, &message),
        raw_response
    );
}
