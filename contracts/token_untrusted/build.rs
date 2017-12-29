
use std::env;
use std::path::{ Path, PathBuf };
use std::process::Command;

fn main () {

  // Gather necessary environment variables
  let rust_sgx_sdk_dir = match env::var("RUST_SGX_SDK") {
    Ok(val) => val,
    Err(_) => panic!("Required environment variable RUST_SGX_SDK not defined")
  };

  let sgx_mode = match env::var("SGX_MODE") {
    Ok(val) => val,
    Err(_) => panic!("Required environment variable SGX_MODE not defined")
  };

  let urts_library_name = match sgx_mode.as_ref() {
    "HW" => "sgx_urts",
    _ => "sgx_urts_sim",
  };

  // Make SGX SDK compiler-rt
  let path_arg = Path::new(&rust_sgx_sdk_dir).join("compiler-rt/");
  let output = Command::new("make")
    .arg("-C")
    .arg(path_arg.to_str().unwrap())
    .output()
    .unwrap();
  assert!(output.status.success());

  // Build Enclave_t.o

  //println!("cargo:rustc-link-search=native=../lib");
  //println!("cargo:rustc-link-lib=static=Enclave_u");

  //println!("cargo:rustc-link-search=native={}/lib64", sgx_sdk_dir);
  //println!("cargo:rustc-link-lib=dylib={}", urts_library_name);

  assert!(false);

}
