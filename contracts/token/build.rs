extern crate protoc_rust;

use std::fs::File;
use std::io::Write;

fn main() {
  protoc_rust::run(protoc_rust::Args {
      out_dir: "src/generated/",
      input: &["src/token_state.proto"],
      includes: &["src/"],
  }).expect("protoc");

  let mut file = File::create("./src/generated/mod.rs").unwrap();
  file.write_all(b"
    pub mod token_state;
  ").unwrap();

}
