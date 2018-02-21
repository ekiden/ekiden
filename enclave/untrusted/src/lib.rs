extern crate sgx_types;
extern crate sgx_urts;

extern crate ekiden_enclave_common;

pub mod enclave;

// Exports.
pub use enclave::Enclave;

// For the below link statements to work, the library paths need to be correctly
// configured. The easiest way to achieve that is to use the build_untrusted
// helper from ekiden_tools.

// Ensure that we link to sgx_urts library.
#[cfg_attr(not(feature = "sgx-simulation"), link(name = "sgx_urts"))]
#[cfg_attr(feature = "sgx-simulation", link(name = "sgx_urts_sim"))]
extern "C" {}
