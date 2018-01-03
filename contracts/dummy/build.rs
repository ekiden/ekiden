extern crate protoc_rust;
extern crate libcontract_utils;

use std::fs::File;
use std::io::Write;

fn main () {
    // TODO: Create a build helper in _utils.
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/generated/",
        input: &["src/dummy.proto"],
        includes: &["src/"],
    }).expect("protoc");

    let mut file = File::create("./src/generated/mod.rs").unwrap();
    file.write_all(b"
        pub mod dummy;
    ").unwrap();

    libcontract_utils::build_trusted();
}
