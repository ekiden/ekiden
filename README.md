# Ekiden

## Dependencies

Here is a brief list of system dependencies currently used for development:
- [rustc](https://www.rust-lang.org/en-US/)
- [cargo](http://doc.crates.io/)
- [docker](https://www.docker.com/)
- [tendermint](https://www.tendermint.com/)
  - Install with [golang](https://golang.org/) `go get github.com/tendermint/tendermint/cmd/tendermint`
- [protoc](https://github.com/google/protobuf/releases)
- [rust-sgx-sdk](https://github.com/baidu/rust-sgx-sdk)

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
$ cargo build -p storage
$ ./target/debug/storage
```

### SGX code

The easiest way to build SGX code is to use the provided scripts, which run a Docker
container with all the included tools.

To start the SGX development container:
```bash
$ ./scripts/rust-sgx-enter.sh
$ cargo build
```

By default, enclaves are built for simulation mode.
Set the following in the `make` invocation to build in the SDK's hardware mode:
```bash
$ export SGX_MODE=HW
```

## Packages
- `abci`: Tendermint Application Blockchain Interface
- `client`: Ekiden client library
- `compute`: Ekiden compute node
- `contracts`: Ekiden contracts (e.g. token)
- `libcontract_common`: source code for `libcontract_*`. Common library for all contracts
- `libcontract_trusted`: packaging for SGX environment
- `libcontract_untrusted`: packaging for non-SGX environment
- `storage`: Ekiden storage node
- `scripts`: Bash scripts for development
- `third_party_sgx`: Forks of third-party packages, with modifications that enable their use with the SGX standard library (`sgx_tstd`).
