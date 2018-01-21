extern crate protobuf;
extern crate sgx_types;
extern crate sgx_urts;

extern crate libcontract_common;

pub mod enclave;
pub mod errors;

// For the below link statements to work, the library paths need to be correctly
// configured. The easiest way to achieve that is to use the build_untrusted
// helper from libcontract_utils.

// Ensure that we link to sgx_urts library.
#[cfg_attr(not(feature = "sgx-simulation"), link(name = "sgx_urts"))]
#[cfg_attr(feature = "sgx-simulation", link(name = "sgx_urts_sim"))]
// Ensure that we link to sgx_uae_service library.
#[cfg_attr(not(feature = "sgx-simulation"), link(name = "sgx_uae_service"))]
#[cfg_attr(feature = "sgx-simulation", link(name = "sgx_uae_service_sim"))]
extern "C" {}
