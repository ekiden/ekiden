extern crate protoc_rust;

use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Command;
//use std::fs::OpenOptions;

fn main() {
  // Gather necessary environment variables
  let rust_sgx_sdk_dir = match env::var("RUST_SGX_SDK") {
    Ok(val) => val,
    Err(_) => panic!("Required environment variable RUST_SGX_SDK not defined")
  };

  let intel_sgx_sdk_dir = match env::var("INTEL_SGX_SDK") {
    Ok(val) => val,
    Err(_) => panic!("Required environment variable INTEL_SGX_SDK not defined")
  };

  let sgx_mode = match env::var("SGX_MODE") {
    Ok(val) => val,
    Err(_) => panic!("Required environment variable SGX_MODE not defined")
  };

  let urts_library_name = match sgx_mode.as_ref() {
    "HW" => "sgx_urts",
    _ => "sgx_urts_sim",
  };

  // Compile .proto files
  protoc_rust::run(protoc_rust::Args {
      out_dir: "src/generated/",
      input: &["src/enclave_rpc.proto"],
      includes: &["src/"],
  }).expect("protoc");

  // For Rust SGX SDK, need to explicitly add some imports
  //let mut file =
  //  OpenOptions::new()
  //  .write(true)
  //  .append(true)
  //  .open("src/generated/enclave_rpc.rs")
  //  .unwrap();
  //writeln!(file, "use std::boxed::Box;").unwrap();

  // Generate a mod.rs for all generated modules
  let mut file = File::create("./src/generated/mod.rs").unwrap();
  file.write_all(b"
    pub mod enclave_rpc;
  ").unwrap();

  // Compile EDL files
  let sgx_edger8r_path = Path::new(&intel_sgx_sdk_dir).join("bin/x64/sgx_edger8r");
  let sgx_include_path = Path::new(&intel_sgx_sdk_dir).join("include");
  let sgx_edl_path = Path::new(&rust_sgx_sdk_dir).join("edl");
  let output = Command::new(sgx_edger8r_path.to_str().unwrap())
    .arg("--trusted")
    .arg("src/Enclave.edl")
    .arg("--search-path")
    .arg(sgx_include_path.to_str().unwrap())
    .arg("--search-path")
    .arg(sgx_edl_path.to_str().unwrap())
    .arg("--trusted-dir")
    .arg("src/generated/trusted/")
    .output()
    .unwrap();
  assert!(output.status.success());
  let output = Command::new(sgx_edger8r_path.to_str().unwrap())
    .arg("--untrusted")
    .arg("src/Enclave.edl")
    .arg("--search-path")
    .arg(sgx_include_path.to_str().unwrap())
    .arg("--search-path")
    .arg(sgx_edl_path.to_str().unwrap())
    .arg("--untrusted-dir")
    .arg("src/generated/untrusted/")
    .output()
    .unwrap();
  assert!(output.status.success());

}
