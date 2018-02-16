pub use ekiden_rpc_untrusted::*;

#[macro_export]
macro_rules! impl_rpc_handler {
    (
        for $struct: ty {
            $( $endpoint: ident => $method: ident, )*
        }
    ) => {
        impl Handler for $struct {
            /// Return a list of endpoints that the handler can handle.
            fn get_endpoints(&self) -> Vec<ClientEndpoint> {
                return vec![
                    $( ClientEndpoint::$endpoint ),*
                ];
            }

            /// Handle a request and return a response.
            fn handle(
                &self,
                endpoint: &ClientEndpoint,
                request: Vec<u8>
            ) -> ::ekiden_core_common::Result<Vec<u8>> {

                use protobuf::parse_from_bytes;

                match *endpoint {
                    $(
                        ClientEndpoint::$endpoint => {
                            Ok(self.$method(parse_from_bytes(&request)?)?.write_to_bytes()?)
                        },
                    )*
                    _ => Err(::ekiden_core_common::Error::new("Invalid RPC router endpoint"))
                }
            }
        }
    }
}
