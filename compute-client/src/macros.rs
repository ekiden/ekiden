#[macro_export]
macro_rules! create_client {
    (
        $api_module: path,

        metadata {
            name = $metadata_name: ident ;
            version = $metadata_version: expr ;
            state_type = $_: ty ;
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
                pub fn new(host: &str, port: u16) -> Result<Self, Error> {
                    let client = ContractClient::new(host, port);

                    // Ensure that the remote server is using the correct contract.
                    let status = client.status()?;
                    if status.contract != stringify!($metadata_name) || status.version != $metadata_version {
                        return Err(Error::new("Server is not running the correct contract"));
                    }

                    Ok(Client {
                        client: client,
                    })
                }

                pub fn status(&self) -> Result<ContractStatus, Error> {
                    self.client.status()
                }

                // Generate methods.
                create_client_methods!( $( $method_name, $request_type, $response_type ),* );
            }
        }
    };
}

/// Internal macro for creating method calls.
#[macro_export]
macro_rules! create_client_methods {
    // Match when no methods are defined.
    () => {};

    // Match each defined method.
    (
        $method_name: ident, $request_type: ty, $response_type: ty
    ) => {
        pub fn $method_name(&self, request: $request_type) -> Result<$response_type, Error> {
            self.client.call(stringify!($method_name), request)
        }
    };

    // Match list of defined methods.
    (
        $method_name: ident, $request_type: ty, $response_type: ty,
        $($x: ident, $y: ty, $z: ty),+
    ) => {
        create_client_methods!($method_name, $request_type, $response_type);
        create_client_methods!($($x, $y, $z),+);
    };
}
