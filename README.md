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
```

To build an enclave (e.g., `dummy` enclave) use the provided script within the container:
```bash
$ ./scripts/build-enclave.sh enclave_dummy
```

To build an example application that uses the `dummy` enclave for testing enclave RPC:
```bash
$ SGX_MODE=SIM cargo build --release -p enclave_test
```

To run the example application:
```bash
$ cd target/release
$ ./enclave_test
```

## Packages
- `abci`: Tendermint Application Blockchain Interface
- `compute`: Ekiden compute node
- `contracts`: Ekiden contracts (e.g. token)
- `storage`: Ekiden storage node
- `scripts`: Bash scripts for development
