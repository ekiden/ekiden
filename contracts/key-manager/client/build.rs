extern crate ekiden_tools;

use std::fs;
use std::io::Write;
use std::process::Command;

fn main() {
    ekiden_tools::generate_mod("src/generated", &[]);

    // Extract MRENCLAVE for key manager contract. If the key manager is not yet
    // available (e.g. because we are building it), use an all-zero value instead.
    let mr_enclave = match Command::new("python")
        .args(&[
            // TODO: Better way to get these paths (env variables).
            "../../../scripts/parse_enclave.py",
            "../../../target/enclave/key-manager.signed.so",
            "--only-mr-enclave",
        ])
        .output()
    {
        Ok(output) => if output.status.success() {
            output.stdout
        } else {
            vec![0u8; 32]
        },
        _ => vec![0u8; 32],
    };

    let mut file = fs::File::create("src/generated/key_manager_mrenclave.bin")
        .expect("Failed to create key manager MRENCLAVE file");

    file.write(&mr_enclave).unwrap();

    println!(
        "cargo:rerun-if-changed={}",
        "../../../scripts/parse_enclave.py"
    );
    println!(
        "cargo:rerun-if-changed={}",
        "../../../target/enclave/key-manager.signed.so"
    );
}
