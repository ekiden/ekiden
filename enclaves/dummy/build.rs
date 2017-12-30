extern crate libenclave_utils;

use std::env;

fn main () {

    // TODO: Improve this.
    let intel_sdk_dir = match env::var("SGX_SDK") {
        Ok(val) => val,
        Err(_) => panic!("Required environment variable SGX_SDK not defined")
    };

    let rust_sdk_dir = "/sgx";

    libenclave_utils::build_trusted(&intel_sdk_dir, &rust_sdk_dir);
}
