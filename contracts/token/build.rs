extern crate protoc_rust;
extern crate libcontract_utils;

use std::env;
use std::fs::File;
use std::io::Write;

fn main() {
  let intel_sgx_sdk_dir = match env::var("INTEL_SGX_SDK") {
    Ok(val) => val,
    Err(_) => panic!("Required environment variable INTEL_SGX_SDK not defined")
  };

  let rust_sgx_sdk_dir = match env::var("RUST_SGX_SDK") {
    Ok(val) => val,
    Err(_) => panic!("Required environment variable RUST_SGX_SDK not defined")
  };

  protoc_rust::run(protoc_rust::Args {
      out_dir: "src/generated/",
      input: &["src/token_state.proto"],
      includes: &["src/"],
  }).expect("protoc");

  let mut file = File::create("./src/generated/mod.rs").unwrap();
  file.write_all(b"
    pub mod token_state;
  ").unwrap();

  libcontract_utils::build_trusted(&intel_sgx_sdk_dir, &rust_sgx_sdk_dir);
}
