use std::env;

extern crate protoc_rust;

fn main () {

    let sdk_dir = match env::var("SGX_SDK") {
        Ok(val) => val,
        Err(_) => panic!("Required environment variable SGX_SDK not defined")
    };

    let mode = match env::var("SGX_MODE") {
        Ok(val) => val,
        Err(_) => panic!("Required environment variable SGX_MODE not defined")
    };

    let urts_library_name = match mode.as_ref() {
        "HW" => "sgx_urts",
        _ => "sgx_urts_sim",
    };

    println!("cargo:rustc-link-search=native=../lib");
    println!("cargo:rustc-link-lib=static=Enclave_u");

    println!("cargo:rustc-link-search=native={}/lib64", sdk_dir);
    println!("cargo:rustc-link-lib=dylib={}", urts_library_name);

    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/",
        input: &["../common/enclave_rpc.proto"],
        includes: &["../common/"],
    }).expect("protoc");
}
