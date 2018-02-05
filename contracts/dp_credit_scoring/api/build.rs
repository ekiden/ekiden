extern crate libcontract_utils;
extern crate protoc;

fn main() {
    libcontract_utils::generate_mod("src/generated", &["api"]);
    libcontract_utils::build_api();

    protoc::run(protoc::Args {
        lang: "python",
        out_dir: "src/generated/",
        input: &["src/api.proto"],
        includes: &["src/"],
        plugin: None,
    }).expect("Failed to run protoc");
}
