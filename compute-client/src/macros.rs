#[macro_export]
macro_rules! create_client {
    (
        $api_module: path,

        metadata {
            name = $metadata_name: ident ;
            version = $metadata_version: expr ;
            state_type = $metadata_state_type: ty ;
            client_attestation_required = $client_attestation_required: expr ;
        }

        $(
            rpc $method_name: ident $method_in: tt -> $method_out: tt ;
        )*
    ) => {
        mod $metadata_name {
            use libcontract_common::quote::MrEnclave;

            use compute_client::*;
            use compute_client::backend::ContractClientBackend;

            pub use $api_module::*;

            pub struct Client<Backend: ContractClientBackend> {
                client: ContractClient<Backend>,
            }

            #[allow(dead_code)]
            impl<Backend: ContractClientBackend> Client<Backend> {
                pub fn new(backend: Backend,
                           mr_enclave: MrEnclave) -> Result<Self, Error> {

                    Ok(Client {
                        client: ContractClient::new(backend, mr_enclave)?,
                    })
                }

                // Generate methods.
                $(
                    create_client_method!($method_name $method_in -> $method_out);
                )*
            }
        }
    };
}

/// Internal macro for creating method calls.
#[macro_export]
macro_rules! create_client_method {
    // State in, state out. E.g., transactions
    ( $method_name: ident ( state , $request_type: ty ) -> ( state , $response_type: ty ) ) => {
        pub fn $method_name(&mut self, request: $request_type) -> Result<$response_type, Error> {
            let response = self.client.call(stringify!($method_name), request)?;
            Ok(response)
        }
    };
    // No state in, state out. E.g., initializers
    ( $method_name: ident ( $request_type: ty ) -> ( state , $response_type: ty ) ) => {
        pub fn $method_name(&mut self, request: $request_type) -> Result<$response_type, Error> {
            let response = self.client.call(stringify!($method_name), request)?;
            Ok(response)
        }
    };
    // State in, no state out. E.g., reads
    ( $method_name: ident ( state , $request_type: ty ) -> $response_type: ty ) => {
        pub fn $method_name(&mut self, request: $request_type) -> Result<$response_type, Error> {
            let response = self.client.call(stringify!($method_name), request)?;
            Ok(response)
        }
    };
    // No state in, no state out. E.g., _metadata
    ( $method_name: ident ( $request_type: ty ) -> $response_type: ty ) => {
        pub fn $method_name(&mut self, request: $request_type) -> Result<$response_type, Error> {
            let response = self.client.call(stringify!($method_name), request)?;
            Ok(response)
        }
    };
}
