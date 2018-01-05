extern crate libcontract_utils;

fn main() {
    libcontract_utils::build_api();

    libcontract_utils::generate_mod("src/generated", &["api"]);

    libcontract_utils::build_trusted();
}
