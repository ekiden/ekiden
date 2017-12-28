extern crate protoc_rust;

use std::fs::File;
use std::io::prelude::*;

fn main() {
  protoc_rust::run(protoc_rust::Args {
      out_dir: "src/generated/",
      input: &["src/enclave_rpc.proto"],
      includes: &["src/"],
  }).expect("protoc");

  let mut file = File::create("./src/generated/mod.rs").unwrap();
  file.write_all(b"pub mod enclave_rpc;").unwrap();

}
