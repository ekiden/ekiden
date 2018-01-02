use sgx_types::*;
use sgx_urts::SgxEnclave;

use std::io::{Read, Write};
use std::fs;
use std::path;
use std::env;

use protobuf;
use protobuf::{Message, MessageStatic};

use generated::enclave_rpc;
use errors;

static ENCLAVE_TOKEN: &'static str = "enclave.token";

extern {
    /// Enclave RPC call API.
    fn rpc_call(eid: sgx_enclave_id_t,
                request_data: *const u8,
                request_length: usize,
                response_data: *const u8,
                response_capacity: usize,
                response_length: *mut usize) -> sgx_status_t;
}

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

        // Step 1: try to retrieve the launch token saved by last transaction
        //         if there is no token, then create a new one.
        //
        // try to get the token saved in $HOME */
        let mut home_dir = path::PathBuf::new();
        let use_token = match env::home_dir() {
            Some(path) => {
                println!("[+] Home dir is {}", path.display());
                home_dir = path;
                true
            },
            None => {
                println!("[-] Cannot get home dir");
                false
            }
        };

        let token_file: path::PathBuf = home_dir.join(ENCLAVE_TOKEN);
        if use_token == true {
            match fs::File::open(&token_file) {
                Err(_) => {
                    println!("[-] Open token file {} error! Will create one.", token_file.as_path().to_str().unwrap());
                },
                Ok(mut f) => {
                    println!("[+] Open token file success! ");
                    match f.read(&mut launch_token) {
                        Ok(1024) => {
                            println!("[+] Token file valid!");
                        },
                        _ => println!("[+] Token file invalid, will create new token file"),
                    }
                }
            }
        }

        // Step 2: call sgx_create_enclave to initialize an enclave instance
        // Debug Support: set 2nd parameter to 1
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

        // Step 3: save the launch token if it is updated
        if use_token == true && launch_token_updated != 0 {
            // reopen the file with write capablity
            match fs::File::create(&token_file) {
                Ok(mut f) => {
                    match f.write_all(&launch_token) {
                        Ok(()) => println!("[+] Saved updated launch token!"),
                        Err(_) => println!("[-] Failed to save updated launch token!"),
                    }
                },
                Err(_) => {
                    println!("[-] Failed to save updated enclave token, but doesn't matter");
                },
            }
        }

        Ok(
            EkidenEnclave {
                enclave: enclave
            }
        )
    }

    /// Destroy the enclave.
    pub fn destroy(self) {
        self.enclave.destroy();
    }

    /// Perform an RPC call against the enclave.
    pub fn call<R: Message, S: Message + MessageStatic>(&self, method: &str, request: &R) -> Result<S, errors::Error> {
        // Prepare request.
        let mut raw_request = enclave_rpc::Request::new();
        raw_request.set_method(String::from(method));
        raw_request.set_payload(request.write_to_bytes()?);

        let raw_response = self.call_raw(&raw_request)?;

        // Deserialize response.
        match protobuf::parse_from_bytes(raw_response.get_payload()) {
            Ok(response) => Ok(response),
            _ => Err(errors::Error::ParseError)
        }
    }

    /// Perform a raw RPC call against the enclave.
    pub fn call_raw(&self, request: &enclave_rpc::Request) -> Result<enclave_rpc::Response, errors::Error> {
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
}
