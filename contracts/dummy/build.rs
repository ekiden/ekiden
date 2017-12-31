extern crate libcontract_utils;

use std::env;

fn main () {

  // TODO: Improve this.
  let intel_sgx_sdk_dir = match env::var("INTEL_SGX_SDK") {
    Ok(val) => val,
    Err(_) => panic!("Required environment variable INTEL_SGX_SDK not defined")
  };

  let rust_sgx_sdk_dir = match env::var("RUST_SGX_SDK") {
    Ok(val) => val,
    Err(_) => panic!("Required environment variable RUST_SGX_SDK not defined")
  };

  libcontract_utils::build_trusted(&intel_sgx_sdk_dir, &rust_sgx_sdk_dir);
}
