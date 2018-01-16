extern crate cc;
extern crate mktemp;
extern crate protoc_rust;

use std::env;
use std::path::Path;
use std::process::Command;
use std::io;
use std::io::prelude::*;
use std::fs;

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

/// Build configuration.
struct BuildConfiguration {
    mode: SgxMode,
    intel_sdk_dir: String,
    rust_sdk_dir: String,
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

/// Get current build environment configuration.
fn get_build_configuration() -> BuildConfiguration {
    // Ensure build script is restarted if any of the env variables changes.
    println!("cargo:rerun-if-env-changed=SGX_MODE");
    println!("cargo:rerun-if-env-changed=INTEL_SGX_SDK");
    println!("cargo:rerun-if-env-changed=RUST_SGX_SDK");

    BuildConfiguration {
        mode: match env::var("SGX_MODE").expect("Please define SGX_MODE").as_ref() {
            "HW" => SgxMode::Hardware,
            _ => SgxMode::Simulation,
        },
        intel_sdk_dir: env::var("INTEL_SGX_SDK").expect("Please define INTEL_SGX_SDK"),
        rust_sdk_dir: env::var("RUST_SGX_SDK").expect("Please define RUST_SGX_SDK"),
    }
}

/// Run edger8r tool from Intel SGX SDK.
fn edger8r(config: &BuildConfiguration, part: BuildPart, output: &str) -> io::Result<()> {
    let edger8r_bin = Path::new(&config.intel_sdk_dir).join(EDGER8R_PATH);

    // Create temporary file with enclave EDL.
    let edl_filename = Path::new(&output).join("enclave.edl");
    let mut edl_file = fs::File::create(&edl_filename)?;
    edl_file.write_all(CONFIG_EKIDEN_EDL.as_bytes())?;

    Command::new(edger8r_bin.to_str().unwrap())
        .args(&["--search-path", Path::new(&config.intel_sdk_dir).join(SGX_SDK_INCLUDE_PATH).to_str().unwrap()])
        .args(&["--search-path", Path::new(&config.rust_sdk_dir).join(RUST_SDK_EDL_PATH).to_str().unwrap()])
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

/// Enable SGX features based on current mode.
pub fn detect_sgx_features() {
    let config = get_build_configuration();

    match config.mode {
        SgxMode::Simulation => {
            // Enable sgx-simulation feature.
            println!("cargo:rustc-cfg=feature=\"sgx-simulation\"");
        },
        _ => {},
    }
}

/// Build the untrusted part of an Ekiden enclave.
pub fn build_untrusted() {
    let config = get_build_configuration();

    /*let urts_library_name = match config.mode {
        SgxMode::Hardware => "sgx_urts",
        SgxMode::Simulation => "sgx_urts_sim",
    };*/

    // Create temporary directory to hold the built libraries.
    let temp_dir = mktemp::Temp::new_dir().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.to_path_buf();
    let temp_dir_name = temp_dir_path.to_str().unwrap();

    // Generate proxy for untrusted part.
    edger8r(&config, BuildPart::Untrusted, &temp_dir_name).expect("Failed to run edger8r");

    // Build proxy.
    cc::Build::new()
        .file(temp_dir_path.join("enclave_u.c"))
        .flag_if_supported("-m64")
        .flag_if_supported("-O0")  // TODO: Should be based on debug/release builds.
        .flag_if_supported("-g")  // TODO: Should be based on debug/release builds.
        .flag_if_supported("-fPIC")
        .flag_if_supported("-Wno-attributes")
        .include(Path::new(&config.intel_sdk_dir).join(SGX_SDK_INCLUDE_PATH))
        .include(&temp_dir_name)
        .compile("enclave_u");

    println!("cargo:rustc-link-lib=static=enclave_u");
    println!("cargo:rustc-link-search=native={}",
             Path::new(&config.intel_sdk_dir).join(SGX_SDK_LIBRARY_PATH).to_str().unwrap());
    // println!("cargo:rustc-link-lib=dylib={}", urts_library_name);
}

/// Build the trusted Ekiden SGX enclave.
pub fn build_trusted() {
    let config = get_build_configuration();

    // Create temporary directory to hold the built libraries.
    let temp_dir = mktemp::Temp::new_dir().expect("Failed to create temporary directory");
    let temp_dir_path = temp_dir.to_path_buf();
    let temp_dir_name = temp_dir_path.to_str().unwrap();

    // Generate proxy for trusted part.
    edger8r(&config, BuildPart::Trusted, &temp_dir_name).expect("Failed to run edger8r");

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
        .include(Path::new(&config.intel_sdk_dir).join(SGX_SDK_INCLUDE_PATH))
        .include(Path::new(&config.intel_sdk_dir).join(SGX_SDK_TLIBC_INCLUDE_PATH))
        .include(Path::new(&config.intel_sdk_dir).join(SGX_SDK_STLPORT_INCLUDE_PATH))
        .include(Path::new(&config.intel_sdk_dir).join(SGX_SDK_EPID_INCLUDE_PATH))
        .include(&temp_dir_name)
        .compile("enclave_t");

    println!("cargo:rustc-link-lib=static=enclave_t");
}

/// Build local contract API files.
pub fn build_api() {
    protoc_rust::run(protoc_rust::Args {
        out_dir: "src/generated/",
        input: &["src/api.proto"],
        includes: &["src/"],
    }).expect("Failed to run protoc");
}

/// Generates a module file with specified exported submodules.
pub fn generate_mod(output_dir: &str, modules: &[&str]) {
    // Create directory if not exist
    fs::create_dir_all(output_dir).unwrap();

    // Create mod.rs
    let output_mod_file = Path::new(&output_dir).join("mod.rs");
    let mut file = fs::File::create(output_mod_file).expect("Failed to create module file");

    for module in modules {
        writeln!(&mut file, "pub mod {};", module).unwrap();
    }

    // Create .gitignore
    let output_gitignore_file = Path::new(&output_dir).join(".gitignore");
    let mut file = fs::File::create(output_gitignore_file).expect("Failed to create .gitignore file");
    writeln!(&mut file, "*").unwrap();
}
