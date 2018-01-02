# Ekiden

## Dependencies

Here is a brief list of system dependencies currently used for development:
- [rustc](https://www.rust-lang.org/en-US/)
- [cargo](http://doc.crates.io/)
- [docker](https://www.docker.com/)
- [rust-sgx-sdk](https://github.com/baidu/rust-sgx-sdk)
  - Clone it to a local directory
- [protoc](https://github.com/google/protobuf/releases)

## Running

The easiest way to run Ekiden is through the provided scripts,
which set up the Docker containers for you.

### Build environment

Currently, the project can only be built in an environment with
the Intel SGX SDK and the Rust SGX SDK.

The easiest way to build SGX code is to use the provided scripts, which run a Docker
container with all the included tools. This has been tested on MacOS and Ubuntu with `SGX_MODE=SIM`.
To enter a Docker environment:
```bash
$ bash scripts/sgx-enter.sh RUST_SGX_SDK_PATH
```

### Storage node

To build and run a storage node:
```bash
$ bash scripts/sgx-enter.sh RUST_SGX_SDK_PATH
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

#### Attaching to an existing container

Currently, the 3 processes (compute, storage, tendermint) look for each other on `localhost`.
In order to attach secondary shells to an existing container, use this helper script:
```bash
$ bash scripts/sgx-attach.sh
```

#### Compiling a contract

By default, enclaves are built for simulation mode.
Set the following in the `make` invocation to build in the SDK's hardware mode:
```bash
$ export SGX_MODE=HW  # default is SGX_MODE=SIM
$ cargo build
```

In order to run a contract on a compute node, we must bundle and sign the contract into an enclave object. For example, to do this for the dummy contract:
```bash
$ bash ./scripts/build-enclave.sh dummy
  ...
  Signed enclave here:
  /code/target/enclave/dummy.signed.so
```

#### Running a contract
The generic compute binary takes a signed contract enclave as a parameter
```bash
$ cargo run -p compute ./target/enclave/dummy.signed.so
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
