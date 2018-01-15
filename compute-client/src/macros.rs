#[macro_export]
macro_rules! create_client {
    (
        $api_module: path,

        metadata {
            name = $metadata_name: ident ;
            version = $metadata_version: expr ;
        }

        $(
            rpc $method_name: ident ( $request_type: ty ) -> $response_type: ty ;
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
                    create_client_method!($method_name, $request_type, $response_type);
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
    ( $method_name: ident, $request_type: ty, $response_type: ty ) => {
        pub fn $method_name(&mut self, request: $request_type) -> Result<$response_type, Error> {
            self.client.call(stringify!($method_name), request)
        }
    };
}
