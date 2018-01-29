extern crate libcontract_utils;
extern crate protoc;

fn main() {
    libcontract_utils::generate_mod("src/generated", &["api"]);
    libcontract_utils::build_api();
}
