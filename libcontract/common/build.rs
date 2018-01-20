extern crate libcontract_utils;
extern crate protoc_rust;

fn main() {
    libcontract_utils::generate_mod("src/generated", &["enclave_rpc"]);

    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/generated/",
        input: &["src/enclave_rpc.proto"],
        includes: &["src/"],
    }).expect("protoc");
}
