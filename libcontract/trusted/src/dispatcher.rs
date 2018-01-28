use sgx_types::*;

use std;
use std::ops::{Deref, DerefMut};

use protobuf;
use protobuf::{Message, MessageStatic};

use libcontract_common::{api, ContractError};
use libcontract_common::client::ClientEndpoint;
use libcontract_common::quote::MrEnclave;

use super::secure_channel::{create_response_box, open_request_box};
use super::untrusted;

/// Wrapper for requests to provide additional request metadata.
pub struct Request<T: Message> {
    /// Underlying request message.
    message: T,
    /// Client short-term public key (if request is authenticated).
    public_key: Option<Vec<u8>>,
    /// Client MRENCLAVE (if channel is mutually authenticated).
    mr_enclave: Option<MrEnclave>,
}

impl<T: Message> Request<T> {
    /// Create new request wrapper from message.
    pub fn new(message: T, public_key: Option<Vec<u8>>, mr_enclave: Option<MrEnclave>) -> Self {
        Request {
            message: message,
            public_key: public_key,
            mr_enclave: mr_enclave,
        }
    }

    /// Copy metadata of the current request into a new request object.
    ///
    /// This method can be used when extracting a part of a request data (e.g. the
    /// payload) and the caller would like to keep the associated metadata. The
    /// metadata will be cloned and the given `message` will be wrapped into a
    /// `Request` object.
    pub fn copy_metadata_to<M: Message>(&self, message: M) -> Request<M> {
        Request {
            message: message,
            public_key: self.public_key.clone(),
            mr_enclave: self.mr_enclave.clone(),
        }
    }

    /// Get short-term public key of the client making this request.
    ///
    /// If the request was made over a non-secure channel, this will be `None`.
    pub fn get_client_public_key(&self) -> &Option<Vec<u8>> {
        &self.public_key
    }

    /// Get MRENCLAVE of the client making this request.
    ///
    /// If the request was made over a channel without client attestation, this
    /// will be `None`.
    pub fn get_client_mr_enclave(&self) -> &Option<MrEnclave> {
        &self.mr_enclave
    }
}

impl<T: Message> Deref for Request<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.message
    }
}

impl<T: Message> DerefMut for Request<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.message
    }
}

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
    api::METHOD_CHANNEL_INIT,
];

/// Parse an RPC request message.
pub fn parse_request(
    request_data: *const u8,
    request_length: usize,
    raw_response: &mut RawResponse,
) -> Result<
    (
        Option<api::CryptoSecretbox>,
        Request<api::PlainClientRequest>,
    ),
    (),
> {
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

    if client_request.has_encrypted_request() {
        // Encrypted request.
        let public_key = client_request
            .get_encrypted_request()
            .get_public_key()
            .to_vec();

        raw_response.public_key = public_key.clone();

        let plain_request = match open_request_box(&client_request.get_encrypted_request()) {
            Ok(plain_request) => plain_request,
            _ => {
                return_error(
                    api::PlainClientResponse_Code::ERROR_SECURE_CHANNEL,
                    "Unable to open secure channel request",
                    &raw_response,
                );
                return Err(());
            }
        };

        Ok((encrypted_state, plain_request))
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

        Ok((encrypted_state, Request::new(plain_request, None, None)))
    }
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
        match create_response_box(&raw_response.public_key, &plain_response) {
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
            let state_bytes = state.write_to_bytes().expect("Failed to serialize state");
            Some(
                super::state_crypto::encrypt_state(state_bytes).expect("Failed to serialize state"),
            )
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

/// Perform an untrusted RPC call against a given (untrusted) endpoint.
///
/// How the actual RPC call is implemented depends on the handler implemented
/// in the untrusted part.
pub fn untrusted_call_endpoint<Rq, Rs>(
    endpoint: &ClientEndpoint,
    request: Rq,
) -> Result<Rs, ContractError>
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
) -> Result<Vec<u8>, ContractError> {
    // Maximum size of serialized response is 16K.
    let mut response: Vec<u8> = Vec::with_capacity(16 * 1024);

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
            return Err(ContractError::new(&format!(
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
