extern crate cc;
extern crate mktemp;

use std::path::Path;
use std::process::Command;
use std::io;
use std::io::prelude::*;
use std::fs::File;

/// SGX build mode.
pub enum SgxMode {
    Hardware,
    Simulation
}

/// Build part.
enum BuildPart {
    Untrusted,
    Trusted,
}

// Paths.
static EDGER8R_PATH: &'static str = "bin/x64/sgx_edger8r";
static SGX_SDK_LIBRARY_PATH: &'static str = "lib64";
static SGX_SDK_INCLUDE_PATH: &'static str = "include";
static SGX_SDK_TLIBC_INCLUDE_PATH: &'static str = "include/tlibc";
static SGX_SDK_STLPORT_INCLUDE_PATH: &'static str = "include/stlport";
static SGX_SDK_EPID_INCLUDE_PATH: &'static str = "include/epid";
static RUST_SDK_EDL_PATH: &'static str = "edl";

// Configuration files.
static CONFIG_EKIDEN_EDL: &'static str = include_str!("../config/ekiden.edl");

/// Run edger8r tool from Intel SGX SDK.
fn edger8r(intel_sdk_dir: &str, rust_sdk_dir: &str, part: BuildPart, output: &str) -> io::Result<()> {
    let edger8r_bin = Path::new(&intel_sdk_dir).join(EDGER8R_PATH);

    // Create temporary file with enclave EDL.
    let edl_filename = Path::new(&output).join("enclave.edl");
    let mut edl_file = File::create(&edl_filename)?;
    edl_file.write_all(CONFIG_EKIDEN_EDL.as_bytes())?;

    Command::new(edger8r_bin.to_str().unwrap())
        .args(&["--search-path", Path::new(&intel_sdk_dir).join(SGX_SDK_INCLUDE_PATH).to_str().unwrap()])
        .args(&["--search-path", Path::new(&rust_sdk_dir).join(RUST_SDK_EDL_PATH).to_str().unwrap()])
        .args(
            &match part {
                BuildPart::Untrusted => ["--untrusted", "--untrusted-dir", &output],
                BuildPart::Trusted => ["--trusted", "--trusted-dir", &output],
            }
        )
        .arg(edl_filename.to_str().unwrap())
        .status()?;

    Ok(())
}

/// Build the untrusted part of an Ekiden enclave.
pub fn build_untrusted(intel_sdk_dir: &str, rust_sdk_dir: &str, mode: SgxMode) {
    let urts_library_name = match mode {
        SgxMode::Hardware => "sgx_urts",
        SgxMode::Simulation => "sgx_urts_sim",
    };

    // Create temporary directory to hold the built libraries.
    let temp_dir = mktemp::Temp::new_dir().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.to_path_buf();
    let temp_dir_name = temp_dir_path.to_str().unwrap();

    // Generate proxy for untrusted part.
    edger8r(&intel_sdk_dir, &rust_sdk_dir, BuildPart::Untrusted, &temp_dir_name)
        .expect("Failed to run edger8r");

    // Build proxy.
    cc::Build::new()
        .file(temp_dir_path.join("enclave_u.c"))
        .flag_if_supported("-m64")
        .flag_if_supported("-O0")  // TODO: Should be based on debug/release builds.
        .flag_if_supported("-g")  // TODO: Should be based on debug/release builds.
        .flag_if_supported("-fPIC")
        .flag_if_supported("-Wno-attributes")
        .include(Path::new(&intel_sdk_dir).join(SGX_SDK_INCLUDE_PATH))
        .include(&temp_dir_name)
        .compile("enclave_u");

    println!("cargo:rustc-link-lib=static=enclave_u");
    println!("cargo:rustc-link-search=native={}",
             Path::new(&intel_sdk_dir).join(SGX_SDK_LIBRARY_PATH).to_str().unwrap());
    println!("cargo:rustc-link-lib=dylib={}", urts_library_name);
}

/// Build the trusted Ekiden SGX enclave.
pub fn build_trusted(intel_sdk_dir: &str, rust_sdk_dir: &str) {
    // Create temporary directory to hold the built libraries.
    let temp_dir = mktemp::Temp::new_dir().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.to_path_buf();
    let temp_dir_name = temp_dir_path.to_str().unwrap();

    // Generate proxy for trusted part.
    edger8r(&intel_sdk_dir, &rust_sdk_dir, BuildPart::Trusted, &temp_dir_name)
        .expect("Failed to run edger8r");

    // Build proxy.
    cc::Build::new()
        .file(temp_dir_path.join("enclave_t.c"))
        .flag_if_supported("-m64")
        .flag_if_supported("-O0")  // TODO: Should be based on debug/release builds.
        .flag_if_supported("-g")  // TODO: Should be based on debug/release builds.
        .flag_if_supported("-nostdinc")
        .flag_if_supported("-fvisibility=hidden")
        .flag_if_supported("-fpie")
        .flag_if_supported("-fstack-protector")
        .include(Path::new(&intel_sdk_dir).join(SGX_SDK_INCLUDE_PATH))
        .include(Path::new(&intel_sdk_dir).join(SGX_SDK_TLIBC_INCLUDE_PATH))
        .include(Path::new(&intel_sdk_dir).join(SGX_SDK_STLPORT_INCLUDE_PATH))
        .include(Path::new(&intel_sdk_dir).join(SGX_SDK_EPID_INCLUDE_PATH))
        .include(&temp_dir_name)
        .compile("enclave_t");

    println!("cargo:rustc-link-lib=static=enclave_t");
}
