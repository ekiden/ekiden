use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::ptr;

use protobuf;
use protobuf::{Message, MessageStatic};

use libcontract_common;
use libcontract_common::api::{Request, PlainRequest, Response, MetadataRequest, MetadataResponse,
                              ContractInitRequest, ContractInitResponse};

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

    /// Perform a plain-text RPC call against the enclave.
    pub fn call<R: Message, S: Message + MessageStatic>(&self, method: &str, request: &R) -> Result<S, errors::Error> {
        // Prepare plain request.
        let mut plain_request = PlainRequest::new();
        plain_request.set_method(String::from(method));
        plain_request.set_payload(request.write_to_bytes()?);

        let mut raw_request = Request::new();
        raw_request.set_plain_request(plain_request);

        let raw_request = raw_request.write_to_bytes()?;
        let raw_response = self.call_raw(&raw_request)?;

        let raw_response: Response = match protobuf::parse_from_bytes(raw_response.as_slice()) {
            Ok(response) => response,
            _ => return Err(errors::Error::ParseError)
        };

        // Plain request requires a plain response.
        assert!(raw_response.has_plain_response());
        let raw_response = raw_response.get_plain_response();

        // Validate response code.
        match raw_response.get_code() {
            libcontract_common::api::PlainResponse_Code::SUCCESS => {},
            code => {
                // Deserialize error.
                let error: libcontract_common::api::Error = match protobuf::parse_from_bytes(raw_response.get_payload()) {
                    Ok(error) => error,
                    _ => return Err(errors::Error::ResponseError(code, "<Unable to parse error payload>".to_string()))
                };

                return Err(errors::Error::ResponseError(code, error.get_message().to_string()))
            }
        };

        // Deserialize response.
        match protobuf::parse_from_bytes(raw_response.get_payload()) {
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
    pub fn get_metadata(&self) -> Result<MetadataResponse, errors::Error> {
        let request = MetadataRequest::new();
        let response: MetadataResponse = self.call("_metadata", &request)?;

        Ok(response)
    }

    /// Perform enclave initialization.
    pub fn initialize(&self, sealed_keys: Vec<u8>) -> Result<ContractInitResponse, errors::Error> {
        let mut request = ContractInitRequest::new();
        request.set_sealed_keys(sealed_keys);

        let response: ContractInitResponse = self.call("_contract_init", &request)?;

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
