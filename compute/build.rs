extern crate ekiden_core_edl;
extern crate ekiden_tools;
extern crate protoc_rust_grpc;

fn main() {
    // Generate module file.
    // Must be done first to create src/generated directory
    ekiden_tools::generate_mod(
        "src/generated",
        &[
            "compute_web3",
            "compute_web3_grpc",
            "consensus",
            "consensus_grpc",
        ],
    );

    protoc_rust_grpc::run(protoc_rust_grpc::Args {
        out_dir: "src/generated/",
        includes: &["src", "../consensus/src"],
        input: &[
            "src/compute_web3.proto",
            "../consensus/src/consensus.proto", // TODO: Move this to a proper location.
        ],
        rust_protobuf: true,
    }).expect("protoc-rust-grpc");

    println!("cargo:rerun-if-changed={}", "src/compute_web3.proto");
    println!(
        "cargo:rerun-if-changed={}",
        "../consensus/src/consensus.proto"
    );

    ekiden_tools::build_untrusted(ekiden_core_edl::edl());
}
