#[macro_export]
macro_rules! create_client {
    (
        $api_module: path,

        metadata {
            name = $metadata_name: ident ;
            version = $metadata_version: expr ;
            state_type = $metadata_state_type: ty ;
        }

        $(
            rpc $method_name: ident $method_in: tt -> $method_out: tt ;
        )*
    ) => {
        mod $metadata_name {
            use compute_client::*;
            pub use $api_module::*;

            pub struct Client {
                client: ContractClient,
            }

            #[allow(dead_code)]
            impl Client {
                pub fn new(host: &str,
                           port: u16,
                           mr_enclave: MrEnclave,
                           ias_config: Option<IASConfiguration>) -> Result<Self, Error> {

                    let mut client = ContractClient::new(host, port, mr_enclave, ias_config)?;

                    // Ensure that the remote server is using the correct contract.
                    let status = client.status()?;
                    if status.contract != stringify!($metadata_name) || status.version != $metadata_version {
                        return Err(Error::new("Server is not running the correct contract"));
                    }

                    // Initialize a secure session.
                    client.init_secure_channel()?;

                    Ok(Client {
                        client: client,
                    })
                }

                pub fn status(&self) -> Result<ContractStatus, Error> {
                    self.client.status()
                }

                // Generate methods.
                $(
                    create_client_method!($method_name $method_in -> $method_out);
                )*
            }

            impl Drop for Client {
                fn drop(&mut self) {
                    // Close secure channel when going out of scope.
                    self.client.close_secure_channel().unwrap();
                }
            }
        }
    };
}

/// Internal macro for creating method calls.
#[macro_export]
macro_rules! create_client_method {
    // State in, state out. E.g., transactions
    ( $method_name: ident ( state , $request_type: ty ) -> ( state , $response_type: ty ) ) => {
        pub fn $method_name(&mut self, state: Vec<u8>, request: $request_type) -> Result<(Vec<u8>, $response_type), Error> {
            let (new_state, response) = self.client.call(stringify!($method_name), Some(state), request)?;
            Ok((new_state.unwrap(), response))
        }
    };
    // No state in, state out. E.g., initializers
    ( $method_name: ident ( state , $request_type: ty ) -> $response_type: ty ) => {
        pub fn $method_name(&mut self, state: Vec<u8>, request: $request_type) -> Result<(Vec<u8>, $response_type), Error> {
            let (_, response) = self.client.call(stringify!($method_name), Some(state), request)?;
            Ok(response)
        }
    };
    // State in, no state out. E.g., reads
    ( $method_name: ident ( $request_type: ty ) -> ( state , $response_type: ty ) ) => {
        pub fn $method_name(&mut self, request: $request_type) -> Result<(Vec<u8>, $response_type), Error> {
            let (new_state, response) = self.client.call(stringify!($method_name), None, request)?;
            Ok((new_state.unwrap(), response))
        }
    };
    // No state in, no state out. E.g., _metadata
    ( $method_name: ident ( $request_type: ty ) -> $response_type: ty ) => {
        pub fn $method_name(&mut self, request: $request_type) -> Result<(Vec<u8>, $response_type), Error> {
            let (_, response) = self.client.call(stringify!($method_name), None, request)?;
            Ok(response)
        }
    };
}
