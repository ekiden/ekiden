#[macro_export]
macro_rules! create_client_rpc {
    (
        $output_module: ident,
        $api_module: path,

        metadata {
            name = $metadata_name: ident ;
            version = $metadata_version: expr ;
            client_attestation_required = $client_attestation_required: expr ;
        }

        $(
            rpc $method_name: ident ( $request_type: ty ) -> $response_type: ty ;
        )*
    ) => {
        mod $output_module {
            use ekiden_rpc_client::*;
            use ekiden_rpc_client::backend::ContractClientBackend;

            pub use $api_module::*;

            pub struct Client<Backend: ContractClientBackend + 'static> {
                client: ContractClient<Backend>,
            }

            #[allow(dead_code)]
            impl<Backend: ContractClientBackend + 'static> Client<Backend> {
                /// Create new client instance.
                pub fn new(backend: Backend,
                           mr_enclave: quote::MrEnclave) -> Self {

                    Client {
                        client: ContractClient::new(
                            backend,
                            mr_enclave,
                            $client_attestation_required,
                        ),
                    }
                }

                /// Initialize a secure channel with the contract.
                ///
                /// If this method is not called, secure channel is automatically initialized
                /// when making the first request.
                pub fn init_secure_channel(&self) -> ClientFuture<()> {
                    self.client.init_secure_channel()
                }

                /// Close secure channel.
                ///
                /// If this method is not called, secure channel is automatically closed in
                /// a blocking fashion when the client is dropped.
                pub fn close_secure_channel(&self) -> ClientFuture<()> {
                    self.client.close_secure_channel()
                }

                // Generate methods.
                $(
                    pub fn $method_name(
                        &mut self,
                        request: $request_type
                    ) -> ClientFuture<$response_type> {
                        self.client.call(stringify!($method_name), request)
                    }
                )*
            }
        }
    };
}
