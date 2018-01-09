use sgx_types::*;
use sgx_urts::SgxEnclave;

use protobuf;
use protobuf::{Message, MessageStatic};

use libcontract_common;
use libcontract_common::api::{MetadataRequest, MetadataResponse, ContractInitRequest, ContractInitResponse};

use errors;

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

    /// Perform an RPC call against the enclave.
    pub fn call<R: Message, S: Message + MessageStatic>(&self, method: &str, request: &R) -> Result<S, errors::Error> {
        // Prepare request.
        let mut raw_request = libcontract_common::api::Request::new();
        raw_request.set_method(String::from(method));
        raw_request.set_payload(request.write_to_bytes()?);

        let raw_response = self.call_raw(&raw_request)?;

        // Validate response code.
        match raw_response.get_code() {
            libcontract_common::api::Response_Code::SUCCESS => {},
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
    pub fn call_raw(&self, request: &libcontract_common::api::Request)
            -> Result<libcontract_common::api::Response, errors::Error> {

        let request = request.write_to_bytes()?;

        // Maximum size of serialized response is 16K.
        let mut response: Vec<u8> = Vec::with_capacity(16 * 1024);

        let mut response_length = 0;
        let status = unsafe {
            rpc_call(
                self.enclave.geteid(),
                request.as_ptr() as * const u8,
                request.len(),
                response.as_ptr() as * const u8,
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

        // Parse response.
        unsafe {
            response.set_len(response_length);
        }

        match protobuf::parse_from_bytes(response.as_slice()) {
            Ok(response) => Ok(response),
            _ => Err(errors::Error::ParseError)
        }
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
