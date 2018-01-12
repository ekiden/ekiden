extern crate protoc_rust_grpc;
extern crate libcontract_utils;

fn main () {
    // Generate module file.
    // Must be done first to create src/generated directory
    libcontract_utils::generate_mod(
        "src/generated",
        &[
            "compute_web3",
            "compute_web3_grpc"
        ]
    );

    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src/generated/",
        includes: &[],
        input: &["src/compute_web3.proto"],
        rust_protobuf: true,
    }).expect("protoc-rust-grpc");

    println!("cargo:rerun-if-changed={}", "src/compute_web3.proto");

    libcontract_utils::build_untrusted();
}
