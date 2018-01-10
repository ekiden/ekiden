use std;

use protobuf;
use protobuf::Message;

use libcontract_common::api;
use errors;

/// Raw data needed to generate the response.
pub struct RawResponse {
    pub data: *mut u8,
    pub capacity: usize,
    pub length: *mut usize,
}

/// Parse an RPC request message.
pub fn parse_request(request_data: *const u8,
                     request_length: usize) -> Result<api::Request, errors::InternalError> {
    let request = unsafe { std::slice::from_raw_parts(request_data, request_length) };
    let request: api::Request = protobuf::parse_from_bytes(request)?;

    Ok(request)
}

/// Serialize and return an RPC response.
pub fn return_response(response: api::Response,
                       raw_response: &RawResponse) {
    let response = response.write_to_bytes().expect("Failed to serialize response");

    // Copy back response.
    if response.len() > raw_response.capacity {
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

/// Serialize and return an RPC success response.
pub fn return_success<S: Message, P: Message>(state: S,
                                              payload: P,
                                              raw_response: &RawResponse) {
    // Prepare response.
    let mut response = api::Response::new();
    response.set_code(api::Response_Code::SUCCESS);

    let state = state.write_to_bytes().expect("Failed to serialize state");
    // TODO: encrypt state
    response.set_state(state);

    let payload = payload.write_to_bytes().expect("Failed to serialize payload");
    response.set_payload(payload);

    return_response(
        response,
        raw_response
    );
}

/// Serialize and return an RPC error response.
pub fn return_error(error: api::Response_Code,
                    message: &str,
                    raw_response: &RawResponse) {
    // Prepare response.
    let mut response = api::Response::new();
    response.set_code(error);

    let mut error = api::Error::new();
    error.set_message(message.to_string());

    let payload = error.write_to_bytes().expect("Failed to serialize error");
    response.set_payload(payload);

    return_response(
        response,
        raw_response
    );
}
