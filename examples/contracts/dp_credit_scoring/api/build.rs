extern crate ekiden_tools;
extern crate protoc;

fn main() {
    ekiden_tools::generate_mod("src/generated", &["api"]);
    ekiden_tools::build_api();

    protoc::run(protoc::Args {
        lang: "python",
        out_dir: "src/generated/",
        input: &["src/api.proto"],
        includes: &["src/"],
        plugin: None,
    }).expect("Failed to run protoc");
}
