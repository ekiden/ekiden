use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::ptr;

use protobuf;
use protobuf::{Message, MessageStatic};

use libcontract_common::api;

use super::errors;

extern {
    /// Enclave RPC call API.
    fn rpc_call(eid: sgx_enclave_id_t,
                request_data: *const u8,
                request_length: usize,
                response_data: *const u8,
                response_capacity: usize,
                response_length: *mut usize) -> sgx_status_t;
}

#[derive(Debug)]
pub struct EkidenEnclave {
    /// Enclave instance.
    enclave: SgxEnclave,
}

impl EkidenEnclave {
    /// Initializes a new Ekiden enclave.
    ///
    /// The created enclave is assumed to implement the Ekiden RPC protocol
    /// via the `rpc_call` method.
    pub fn new(filename: &str) -> Result<Self, errors::Error> {
        let mut launch_token: sgx_launch_token_t = [0; 1024];
        let mut launch_token_updated: i32 = 0;

        // Initialize enclave.
        let debug = 1;
        let mut misc_attr = sgx_misc_attribute_t {
            secs_attr: sgx_attributes_t {
                flags: 0,
                xfrm: 0
            },
            misc_select: 0
        };

        let enclave = match SgxEnclave::create(
            filename,
            debug,
            &mut launch_token,
            &mut launch_token_updated,
            &mut misc_attr
        ) {
            Ok(enclave) => enclave,
            Err(_) => { return Err(errors::Error::SgxError); }
        };

        Ok(
            EkidenEnclave {
                enclave: enclave
            }
        )
    }

    /// Perform a plain-text RPC call against the enclave with no state.
    pub fn call<R: Message, S: Message + MessageStatic>(&self, method: &str, request: &R) -> Result<S, errors::Error> {
        // Prepare plain request.
        let mut plain_request = api::PlainClientRequest::new();
        plain_request.set_method(String::from(method));
        plain_request.set_payload(request.write_to_bytes()?);

        let mut client_request = api::ClientRequest::new();
        client_request.set_plain_request(plain_request);

        let mut enclave_request = api::EnclaveRequest::new();
        enclave_request.set_client_request(client_request);

        let enclave_request_bytes = enclave_request.write_to_bytes()?;
        let enclave_response_bytes = self.call_raw(&enclave_request_bytes)?;

        let enclave_response: api::EnclaveResponse = match protobuf::parse_from_bytes(enclave_response_bytes.as_slice()) {
            Ok(enclave_response) => enclave_response,
            _ => return Err(errors::Error::ParseError)
        };

        let client_response = enclave_response.get_client_response();

        // Plain request requires a plain response.
        assert!(client_response.has_plain_response());
        let plain_response = client_response.get_plain_response();

        // Validate response code.
        match plain_response.get_code() {
            api::PlainClientResponse_Code::SUCCESS => {},
            code => {
                // Deserialize error.
                let error: api::Error = match protobuf::parse_from_bytes(plain_response.get_payload()) {
                    Ok(error) => error,
                    _ => return Err(errors::Error::ResponseError(code, "<Unable to parse error payload>".to_string()))
                };

                return Err(errors::Error::ResponseError(code, error.get_message().to_string()))
            }
        };

        // Deserialize response.
        match protobuf::parse_from_bytes(plain_response.get_payload()) {
            Ok(response) => Ok(response),
            _ => Err(errors::Error::ParseError)
        }
    }

    /// Perform a raw RPC call against the enclave.
    pub fn call_raw(&self, request: &Vec<u8>) -> Result<Vec<u8>, errors::Error> {
        // Maximum size of serialized response is 16K.
        let mut response: Vec<u8> = Vec::with_capacity(16 * 1024);

        let mut response_length = 0;
        let status = unsafe {
            rpc_call(
                self.enclave.geteid(),
                request.as_ptr() as * const u8,
                request.len(),
                response.as_mut_ptr() as * mut u8,
                response.capacity(),
                &mut response_length,
            )
        };

        match status {
            sgx_status_t::SGX_SUCCESS => {},
            _ => {
                return Err(errors::Error::SgxError);
            }
        }

        unsafe {
            response.set_len(response_length);
        }

        Ok(response)
    }

    /// Returns enclave metadata.
    pub fn get_metadata(&self) -> Result<api::MetadataResponse, errors::Error> {
        let request = api::MetadataRequest::new();
        let response: api::MetadataResponse = self.call("_metadata", &request)?;

        Ok(response)
    }

    /// Perform enclave initialization.
    pub fn initialize(&self) -> Result<api::ContractInitResponse, errors::Error> {
        let request = api::ContractInitRequest::new();
        let response: api::ContractInitResponse = self.call("_contract_init", &request)?;

        Ok(response)
    }

    /// Restore enclave from previous initialization.
    pub fn restore(&self, sealed_keys: Vec<u8>) -> Result<api::ContractRestoreResponse, errors::Error> {
        let mut request = api::ContractRestoreRequest::new();
        request.set_sealed_keys(sealed_keys);

        let response: api::ContractRestoreResponse = self.call("_contract_restore", &request)?;

        Ok(response)
    }
}

#[no_mangle]
pub extern "C" fn untrusted_init_quote(p_target_info: *mut sgx_target_info_t,
                                       p_gid: *mut sgx_epid_group_id_t) -> sgx_status_t {

    unsafe {
        sgx_init_quote(p_target_info, p_gid)
    }
}

#[no_mangle]
pub extern "C" fn untrusted_get_quote(p_report: *const sgx_report_t,
                                      quote_type: sgx_quote_sign_type_t,
                                      p_spid: *const sgx_spid_t,
                                      p_nonce: *const sgx_quote_nonce_t,
                                      p_qe_report: *mut sgx_report_t,
                                      p_quote: *mut u8,
                                      _quote_capacity: u32,
                                      quote_size: *mut u32) -> sgx_status_t {
    // Calculate quote size.
    let status = unsafe {
        sgx_calc_quote_size(
            ptr::null(),
            0,
            quote_size
        )
    };

    match status {
        sgx_status_t::SGX_SUCCESS => {},
        _ => return status
    };

    // Get quote from the quoting enclave.
    unsafe {
        sgx_get_quote(
            p_report,
            quote_type,
            p_spid,
            p_nonce,
            ptr::null(),
            0,
            p_qe_report,
            p_quote as *mut sgx_quote_t,
            *quote_size
        )
    }
}
