extern crate ekiden_tools;

fn main() {
    ekiden_tools::generate_mod("src/generated", &["enclave_rpc", "enclave_services"]);

    ekiden_tools::protoc(ekiden_tools::ProtocArgs {
        out_dir: "src/generated/",
        input: &["src/enclave_rpc.proto", "src/enclave_services.proto"],
        includes: &["src/"],
    });
}
