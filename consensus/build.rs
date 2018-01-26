extern crate libcontract_utils;
extern crate protoc_rust_grpc;

fn main() {
    // Generate module file.
    // Must be done first to create src/generated directory
    libcontract_utils::generate_mod("src/generated", &["consensus", "consensus_grpc"]);

    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src/generated/",
        includes: &[],
        input: &["src/consensus.proto"],
        rust_protobuf: true,
    }).expect("protoc-rust-grpc");
}
