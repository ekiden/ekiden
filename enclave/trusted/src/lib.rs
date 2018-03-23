#[cfg(target_env = "sgx")]
extern crate sgx_trts;

extern crate ekiden_common;

pub mod utils;

/// Declare enclave initialization structures.
///
/// **This macro must be used in each enclave in order for the initialization
/// handlers of other modules to work correctly.***
#[macro_export]
macro_rules! enclave_init {
    () => {
        #[doc(hidden)]
        #[no_mangle]
        pub extern "C" fn __ekiden_enclave() {
            // We define a symbol called __ekiden_enclave, which is forced to be
            // used by the linker script. Without this, the .init_array section
            // of the resulting library is removed by the linker and thus no
            // initialization is done.
        }
    }
}
