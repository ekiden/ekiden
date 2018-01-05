extern crate protoc_rust_grpc;
extern crate libcontract_utils;

use std::fs::File;
use std::io::Write;

fn main () {
  protoc_rust_grpc::run(protoc_rust_grpc::Args {
    out_dir: "src/generated/",
    includes: &[],
    input: &["src/storage.proto"],
    rust_protobuf: true,
  }).expect("protoc-rust-grpc");

  let mut file = File::create("./src/generated/mod.rs").unwrap();
  file.write_all(b"
    pub mod storage;
    pub mod storage_grpc;
  ").unwrap();

}
