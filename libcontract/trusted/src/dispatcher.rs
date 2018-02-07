use sgx_types::*;

use std;
use std::ops::{Deref, DerefMut};

use protobuf;
use protobuf::{Message, MessageStatic};

use libcontract_common::{api, ContractError};
use libcontract_common::client::ClientEndpoint;
use libcontract_common::quote::MrEnclave;

use super::errors::DispatchError;
use super::secure_channel::{create_response_box, open_request_box};
use super::untrusted;

/// Wrapper for requests to provide additional request metadata.
pub struct Request<T: Message + MessageStatic> {
    /// Underlying request message.
    message: T,
    /// Client short-term public key (if request is authenticated).
    public_key: Option<Vec<u8>>,
    /// Client MRENCLAVE (if channel is mutually authenticated).
    mr_enclave: Option<MrEnclave>,
    /// Optional error occurred during request processing.
    error: Option<DispatchError>,
}

impl<T: Message + MessageStatic> Request<T> {
    /// Create new request wrapper from message.
    pub fn new(message: T, public_key: Option<Vec<u8>>, mr_enclave: Option<MrEnclave>) -> Self {
        Request {
            message: message,
            public_key: public_key,
            mr_enclave: mr_enclave,
            error: None,
        }
    }

    /// Create new request with dispatch error.
    pub fn error(error: DispatchError) -> Self {
        Request {
            message: T::new(),
            public_key: None,
            mr_enclave: None,
            error: Some(error),
        }
    }

    /// Copy metadata of the current request into a new request object.
    ///
    /// This method can be used when extracting a part of a request data (e.g. the
    /// payload) and the caller would like to keep the associated metadata. The
    /// metadata will be cloned and the given `message` will be wrapped into a
    /// `Request` object.
    pub fn copy_metadata_to<M: Message + MessageStatic>(&self, message: M) -> Request<M> {
        Request {
            message: message,
            public_key: self.public_key.clone(),
            mr_enclave: self.mr_enclave.clone(),
            error: None,
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

    /// Get optional error if any occurred during dispatch.
    pub fn get_error(&self) -> &Option<DispatchError> {
        &self.error
    }
}

impl<T: Message + MessageStatic> Deref for Request<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.message
    }
}

impl<T: Message + MessageStatic> DerefMut for Request<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.message
    }
}

/// Wrapper for responses.
pub struct Response<State> {
    /// Response message.
    message: api::ClientResponse,
    /// Optional state.
    state: Option<State>,
}

impl<State> Response<State> {
    /// Create new response.
    pub fn new<Rq>(request: &Request<Rq>, response: api::PlainClientResponse) -> Self
    where
        Rq: Message + MessageStatic,
    {
        let mut message = api::ClientResponse::new();
        if let &Some(ref public_key) = request.get_client_public_key() {
            // Encrypted response.
            match create_response_box(&public_key, &response) {
                Ok(response_box) => message.set_encrypted_response(response_box),
                _ => {
                    // Failed to create a cryptographic box for the response. This could
                    // be due to the session being incorrect or due to other issues. In
                    // this case, we should generate a plain error message.
                    message.set_plain_response(Self::generate_error(
                        api::PlainClientResponse_Code::ERROR_SECURE_CHANNEL,
                        "Failed to generate secure channel response",
                    ));
                }
            };
        } else {
            // Plain response.
            message.set_plain_response(response);
        }

        Response {
            message,
            state: None,
        }
    }

    /// Create success response.
    pub fn success<Rq, Rs>(request: &Request<Rq>, payload: Rs) -> Self
    where
        Rq: Message + MessageStatic,
        Rs: Message + MessageStatic,
    {
        // Prepare response.
        let mut response = api::PlainClientResponse::new();
        response.set_code(api::PlainClientResponse_Code::SUCCESS);

        let payload = payload
            .write_to_bytes()
            .expect("Failed to serialize payload");
        response.set_payload(payload);

        Self::new(&request, response)
    }

    /// Create error response.
    pub fn error<Rq>(
        request: &Request<Rq>,
        error: api::PlainClientResponse_Code,
        message: &str,
    ) -> Self
    where
        Rq: Message + MessageStatic,
    {
        Self::new(&request, Self::generate_error(error, &message))
    }

    /// Generate error response.
    fn generate_error(
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

    /// Take response message.
    ///
    /// After calling this method, a default message will be left in its place.
    pub fn take_message(&mut self) -> api::ClientResponse {
        std::mem::replace(&mut self.message, api::ClientResponse::new())
    }

    /// Take returned state from response.
    ///
    /// After calling this method, `None` will be left in place of state.
    pub fn take_state(&mut self) -> Option<State> {
        self.state.take()
    }

    /// Adds state modification to this response.
    pub fn with_state(mut self, state: State) -> Self {
        self.state = Some(state);

        self
    }
}

/// List of methods that allow plain requests. All other requests must be done over
/// a secure channel.
const PLAIN_METHODS: &'static [&'static str] = &[
    "_metadata",
    "_contract_init",
    "_contract_restore",
    api::METHOD_CHANNEL_INIT,
    api::METHOD_STATE_DIFF,
    api::METHOD_STATE_APPLY,
];

/// Parse an RPC request message.
pub fn parse_request(
    request_data: *const u8,
    request_length: usize,
) -> Result<
    (
        Option<api::CryptoSecretbox>,
        Vec<Request<api::PlainClientRequest>>,
    ),
    (),
> {
    let raw_request = unsafe { std::slice::from_raw_parts(request_data, request_length) };
    let mut enclave_request: api::EnclaveRequest = match protobuf::parse_from_bytes(raw_request) {
        Ok(enclave_request) => enclave_request,
        _ => {
            // Malformed outer request, enclave will panic.
            panic!("Malformed enclave request");
        }
    };

    let encrypted_state = if enclave_request.has_encrypted_state() {
        Some(enclave_request.take_encrypted_state())
    } else {
        None
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
            let plain_request = client_request.take_plain_request();
            let plain_request = match PLAIN_METHODS
                .iter()
                .find(|&method| method == &plain_request.get_method())
            {
                Some(_) => Request::new(plain_request, None, None),
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

    Ok((encrypted_state, requests))
}

/// Serialize and return an RPC response.
pub fn return_response<State>(
    encrypted_state: Option<api::CryptoSecretbox>,
    responses: Vec<Response<State>>,
    response_data: *mut u8,
    response_capacity: usize,
    response_length: *mut usize,
) {
    let mut enclave_response = api::EnclaveResponse::new();

    // Add encrypted state.
    if let Some(encrypted_state) = encrypted_state {
        enclave_response.set_encrypted_state(encrypted_state);
    }

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
