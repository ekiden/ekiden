extern crate protoc_rust_grpc;

extern crate libcontract_utils;

fn main() {
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
        includes: &["../compute/src/"],
        input: &["../compute/src/compute_web3.proto"],  // TODO: Move this to a proper location.
        rust_protobuf: true,
    }).expect("protoc-rust-grpc");

    println!("rerun-if-changed=../compute/src/compute_web3.proto");

}
