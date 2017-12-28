extern crate protoc_rust_grpc;

use std::fs::File;
use std::io::Write;

fn main() {
  protoc_rust_grpc::run(protoc_rust_grpc::Args {
    out_dir: "src/generated/",
    includes: &[],
    input: &["src/ekiden_web3.proto"],
    rust_protobuf: true,
  }).expect("protoc-rust-grpc");

  let mut file = File::create("./src/generated/mod.rs").unwrap();
  file.write_all(b"
    pub mod ekiden_web3;
    pub mod ekiden_web3_grpc;
  ").unwrap();
}
