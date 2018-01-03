# Ekiden

## Dependencies

Here is a brief list of system dependencies currently used for development:
- [rustc](https://www.rust-lang.org/en-US/)
- [cargo](http://doc.crates.io/)
- [cargo-make](https://crates.io/crates/cargo-make)
- [docker](https://www.docker.com/)
- [tendermint](https://www.tendermint.com/)
  - Install with [golang](https://golang.org/) `go get github.com/tendermint/tendermint/cmd/tendermint`
- [protoc](https://github.com/google/protobuf/releases)
- [rust-sgx-sdk](https://github.com/baidu/rust-sgx-sdk)

## Building

The easiest way to build SGX code is to use the provided scripts, which run a Docker
container with all the included tools. This has been tested on MacOS and Ubuntu with `SGX_MODE=SIM`.

To start the SGX development container:
```bash
$ ./scripts/rust-sgx-enter.sh
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

### Tendermint

The easiest way to run Tendermint is to use the provided scripts, which run the Docker
containers for you.

To start a Tendermint node:
```bash
$ ./scripts/tendermint_start
```

To clear all data:
```bash
$ ./scripts/tendermint_clear
```

### Storage node

To build and run a storage node:
```bash
$ ./target/debug/storage
```

### Compute node

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
