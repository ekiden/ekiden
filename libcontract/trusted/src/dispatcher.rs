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
pub fn parse_request(
    request_data: *const u8,
    request_length: usize,
    raw_response: &mut RawResponse,
) -> Result<(Option<api::CryptoSecretbox>, api::PlainClientRequest), ()> {
    let raw_request = unsafe { std::slice::from_raw_parts(request_data, request_length) };
    let mut enclave_request: api::EnclaveRequest = match protobuf::parse_from_bytes(raw_request) {
        Ok(enclave_request) => enclave_request,
        _ => {
            return_error(
                api::PlainClientResponse_Code::ERROR_BAD_REQUEST,
                "Unable to parse request",
                &raw_response,
            );
            return Err(());
        }
    };

    let encrypted_state = if enclave_request.has_encrypted_state() {
        Some(enclave_request.take_encrypted_state())
    } else {
        None
    };

    let mut client_request = enclave_request.take_client_request();

    let plain_request = if client_request.has_encrypted_request() {
        // Encrypted request.
        raw_response.public_key = client_request
            .get_encrypted_request()
            .get_public_key()
            .to_vec();
        match secure_channel::open_request_box(&client_request.get_encrypted_request()) {
            Ok(plain_request) => plain_request,
            _ => {
                return_error(
                    api::PlainClientResponse_Code::ERROR_SECURE_CHANNEL,
                    "Unable to open secure channel request",
                    &raw_response,
                );
                return Err(());
            }
        }
    } else {
        // Plain request.
        let plain_request = client_request.take_plain_request();
        match PLAIN_METHODS
            .iter()
            .find(|&method| method == &plain_request.get_method())
        {
            Some(_) => {}
            None => {
                // Method requires a secure channel.
                return_error(
                    api::PlainClientResponse_Code::ERROR_METHOD_SECURE,
                    "Method call must be made over a secure channel",
                    &raw_response,
                );
                return Err(());
            }
        };

        plain_request
    };

    Ok((encrypted_state, plain_request))
}

/// Serialize and return an RPC response.
pub fn return_response(
    encrypted_state: Option<api::CryptoSecretbox>,
    plain_response: api::PlainClientResponse,
    raw_response: &RawResponse,
) {
    let mut enclave_response = api::EnclaveResponse::new();

    if let Some(encrypted_state) = encrypted_state {
        enclave_response.set_encrypted_state(encrypted_state);
    }

    let mut client_response = api::ClientResponse::new();
    if raw_response.public_key.is_empty() {
        // Plain response.
        client_response.set_plain_response(plain_response);
    } else {
        // Encrypted response.
        match secure_channel::create_response_box(&raw_response.public_key, &plain_response) {
            Ok(response_box) => client_response.set_encrypted_response(response_box),
            _ => {
                // Failed to create a cryptographic box for the response. This could
                // be due to the session being incorrect or due to other issues. In
                // this case, we should generate a plain error message.
                client_response.set_plain_response(generate_error(
                    api::PlainClientResponse_Code::ERROR_SECURE_CHANNEL,
                    "Failed to generate secure channel response",
                ));
            }
        };
    }
    enclave_response.set_client_response(client_response);

    // TODO: Return null response instead?
    let enclave_response_bytes = enclave_response
        .write_to_bytes()
        .expect("Failed to serialize response");

    // Copy back response.
    if enclave_response_bytes.len() > raw_response.capacity {
        // TODO: Return null response instead?
        panic!("Not enough space for response.");
    } else {
        unsafe {
            for i in 0..enclave_response_bytes.len() as isize {
                std::ptr::write(
                    raw_response.data.offset(i),
                    enclave_response_bytes[i as usize],
                );
            }
            *raw_response.length = enclave_response_bytes.len();
        };
    }
}

/// Generate error response.
pub fn generate_error(
    error: api::PlainClientResponse_Code,
    message: &str,
) -> api::PlainClientResponse {
    // Prepare response.
    let mut response = api::PlainClientResponse::new();
    response.set_code(error);

    let mut error = api::Error::new();
    error.set_message(message.to_string());

    let payload = error.write_to_bytes().expect("Failed to serialize error");
    response.set_payload(payload);

    response
}

/// Serialize and return an RPC success response.
pub fn return_success<S: Message, P: Message>(
    state: Option<S>,
    payload: P,
    raw_response: &RawResponse,
) {
    // Prepare response.
    let mut response = api::PlainClientResponse::new();
    response.set_code(api::PlainClientResponse_Code::SUCCESS);

    let payload = payload
        .write_to_bytes()
        .expect("Failed to serialize payload");
    response.set_payload(payload);

    let encrypted_state = match state {
        Some(state) => {
            Some(super::state_crypto::encrypt_state(&state).expect("Failed to serialize state"))
        }
        None => None,
    };

    return_response(encrypted_state, response, raw_response);
}

/// Serialize and return an RPC error response.
pub fn return_error(
    error: api::PlainClientResponse_Code,
    message: &str,
    raw_response: &RawResponse,
) {
    return_response(None, generate_error(error, &message), raw_response);
}
