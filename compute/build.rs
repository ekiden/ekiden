extern crate libcontract_utils;
extern crate protoc_rust_grpc;

fn main() {
    // Generate module file.
    // Must be done first to create src/generated directory
    libcontract_utils::generate_mod(
        "src/generated",
        &[
            "compute_web3",
            "compute_web3_grpc",
            "storage",
            "storage_grpc",
        ],
    );

    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src/generated/",
        includes: &["src", "../storage/src"],
        input: &[
            "src/compute_web3.proto",
            "../storage/src/storage.proto", // TODO: Move this to a proper location.
        ],
        rust_protobuf: true,
    }).expect("protoc-rust-grpc");

    println!("cargo:rerun-if-changed={}", "src/compute_web3.proto");
    println!("cargo:rerun-if-changed={}", "../storage/src/storage.proto");

    libcontract_utils::build_untrusted();
}
