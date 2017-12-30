extern crate libenclave_utils;

use std::env;

use libenclave_utils::SgxMode;

fn main () {

    // TODO: Improve this.
    let intel_sdk_dir = match env::var("SGX_SDK") {
        Ok(val) => val,
        Err(_) => panic!("Required environment variable SGX_SDK not defined")
    };

    let rust_sdk_dir = "/sgx";

    let mode = match env::var("SGX_MODE") {
        Ok(val) => match val.as_ref() {
            "HW" => SgxMode::Hardware,
            _ => SgxMode::Simulation,
        },
        Err(_) => panic!("Required environment variable SGX_MODE not defined")
    };

    libenclave_utils::build_untrusted(&intel_sdk_dir, &rust_sdk_dir, mode);
}
