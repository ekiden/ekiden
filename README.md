# Ekiden

## Dependencies

Here is a brief list of system dependencies currently used for development:
- [rustc](https://www.rust-lang.org/en-US/)
- [cargo](http://doc.crates.io/)
- [cargo-make](https://crates.io/crates/cargo-make)
- [docker](https://www.docker.com/)
- [rust-sgx-sdk](https://github.com/baidu/rust-sgx-sdk)
  - Clone it to a local directory
- [protoc](https://github.com/google/protobuf/releases)

## Building

The easiest way to build SGX code is to use the provided scripts, which run a Docker
container with all the included tools. This has been tested on MacOS and Ubuntu with `SGX_MODE=SIM`.

To start the SGX development container:
```bash
$ ./scripts/sgx-enter.sh RUST_SGX_SDK_PATH
```

Ekiden uses [`cargo-make`](https://crates.io/crates/cargo-make) as the build system. To install it,
run:
```bash
$ cargo install cargo-make
```

Then, to build everything required for running Ekiden, simply run the following in the top-level
directory:
```bash
$ cargo make
```

This should install any required dependencies and build all packages. By default SGX code is
built in simulation mode. To change this, do `export SGX_MODE=HW` (currently untested) before
running the `cargo make` command.

## Running

The easiest way to run Ekiden is through the provided scripts,
which set up the Docker containers for you.

### Storage node

To build and run a storage node:
```bash
$ bash scripts/sgx-enter.sh 
$ cargo run -p storage
```

The storage node depends on a local instance of Tendermint
To start a Tendermint docker container that is linked to the container above:
```bash
$ bash ./scripts/tendermint-start.sh
```

Occasionally, you'll need to clear all persistent data. To clear all data:
```bash
$ bash ./scripts/tendermint-clear.sh
```

### Compute node

Currently, the 3 processes (compute, storage, tendermint) look for each other on `localhost`.
In order to attach secondary shells to an existing container, use this helper script:
```bash
$ bash scripts/sgx-attach.sh
```

The generic compute binary takes a signed contract enclave as a parameter:
```bash
$ cargo run -p compute ./target/enclave/dummy.signed.so
```

To get a list of built enclaves:
```bash
$ ls ./target/enclave/*.signed.so
```

## Packages
- `abci`: Tendermint Application Blockchain Interface
- `client`: Ekiden client library
- `compute`: Ekiden compute node
- `contracts`: Ekiden contracts (e.g. token)
- `libcontract/common`: common library for all Ekiden contracts
  - source code directory for `libcontract_*`.
- `libcontract/trusted`: `libcontract` packaging for SGX environment
- `libcontract/untrusted`: `libcontract` packaging for non-SGX environment
- `libcontract/utils`: Utilities for easier builds with SGX enclaves
- `storage`: Ekiden storage node
- `scripts`: Bash scripts for development
- `third_party`: Forks of third-party packages, with modifications that enable their use with the SGX standard library (`sgx_tstd`).
