/// Macro for creating API definitions.
///
/// This is a meta-macro, which actually defines two macros that get passed
/// the same API definition. One is for use inside enclaves and the other is
/// for use inside clients.
#[macro_export]
macro_rules! contract_api {
    ($($api: tt)*) => {
        /// Macro for use inside enclaves.
        #[macro_export]
        macro_rules! create_enclave_api {
            () => {
                create_enclave! { $($api)* }
            }
        }

        /// Macro for use to generate clients.
        #[macro_export]
        macro_rules! create_client_api {
            () => {
                create_client! { $crate, $($api)* }
            }
        }
    }
}
