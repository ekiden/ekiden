# Ekiden

## Dependencies

Here is a brief list of system dependencies currently used for development:
- [Rust](https://www.rust-lang.org/en-US/)
- [Cargo](http://doc.crates.io/)
- [Docker](https://www.docker.com/)
- [Tendermint](https://www.tendermint.com/)
  - Install with [golang](https://golang.org/) `go get github.com/tendermint/tendermint/cmd/tendermint`

## Running

To get Tendermint running:

```bash
  $ tendermint init   # This only has to run once
  $ tendermint node
```

To build and run a storage node:
```bash
  $ cargo build
  $ ./target/debug/storage
```

## Packages
- `abci`: Tendermint Application Blockchain Interface
- `compute`: Ekiden compute node
- `contracts`: Ekiden contracts (e.g. token)
- `storage`: Ekiden storage node
- `scripts`: Bash scripts for development
