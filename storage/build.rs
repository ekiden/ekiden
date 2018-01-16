extern crate protoc_rust_grpc;
extern crate libcontract_utils;

use std::fs::File;
use std::io::Write;

fn main () {
  // Generate module file.
  // Must be done first to create src/generated directory
  libcontract_utils::generate_mod(
    "src/generated",
    &[
      "storage",
      "storage_grpc",
    ]
  );

  protoc_rust_grpc::run(protoc_rust_grpc::Args {
    out_dir: "src/generated/",
    includes: &[],
    input: &["src/storage.proto"],
    rust_protobuf: true,
  }).expect("protoc-rust-grpc");

}
