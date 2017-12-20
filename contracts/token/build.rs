extern crate protoc_rust;

fn main() {
  protoc_rust::run(protoc_rust::Args {
      out_dir: "src/",
      input: &["src/token_state.proto"],
      includes: &["src/"],
  }).expect("protoc");
}
