extern crate cc;

use std::env;
use std::path::Path;
use std::process::Command;

fn main () {
  // Gather necessary environment variables
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

  ///////////////////////////
  // BUILD main.rs
  ///////////////////////////

  // Build Enclave_u.o
  // NOTE: libcontract_trusted must be a build-dependency for Enclave_u.c to exist
  let sgx_include_path = Path::new(&intel_sgx_sdk_dir).join("include");
  let include_path = Path::new("../libcontract_trusted/src/generated/untrusted/");
  let src_path = Path::new("../libcontract_trusted/src/generated/untrusted/Enclave_u.c");
  cc::Build::new()
    .file(src_path.to_str().unwrap())
    .flag_if_supported("-m64")
    .flag_if_supported("-O0")
    .flag_if_supported("-g")
    .flag_if_supported("-fPIC")
    .flag_if_supported("-Wno-attributes")
    .include(sgx_include_path.to_str().unwrap())
    .include(include_path.to_str().unwrap())
    .compile("Enclave_u");

  //ar rcsD app/libEnclave_u.a app/Enclave_u.o
  //println!("cargo:rustc-link-search=native=../lib");
  println!("cargo:rustc-link-lib=static=Enclave_u");
  println!("cargo:rustc-link-search=native={}/lib64", intel_sgx_sdk_dir);
  println!("cargo:rustc-link-lib=dylib={}", urts_library_name);

}
