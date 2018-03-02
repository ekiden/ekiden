use sgx_types::*;

use protobuf::{self, Message, MessageStatic};

use ekiden_common::error::{Error, Result};
use ekiden_rpc_common::client::ClientEndpoint;

use super::untrusted;

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
