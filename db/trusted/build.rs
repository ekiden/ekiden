extern crate ekiden_tools;
extern crate protoc_rust;

fn main() {
    ekiden_tools::generate_mod("src/generated", &["database"]);

    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/generated/",
        input: &["src/database.proto"],
        includes: &["src/"],
    }).expect("protoc");
}
