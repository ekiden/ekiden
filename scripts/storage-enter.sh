#!/bin/bash

CWD=$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )
rust_sgx_sdk_dir=${1:-${CWD}/../rust-sgx-sdk}

which docker >/dev/null || {
  echo "ERROR: Please install Docker first."
  exit 1
}

[ -d ${rust_sgx_sdk_dir} ] || {
  echo "ERROR: Please checkout rust-sgx-sdk into the following directory:"
  echo "  ${rust_sgx_sdk_dir}"
  echo ""
  echo "Or provide the correct directory as an argument to this script."
  exit 1
}

# Start SGX Rust Docker container.
docker run --rm -t -i \
  --name "storage" \
  -v ${CWD}:/code \
  -v ${rust_sgx_sdk_dir}:/sgx \
  -e "SGX_MODE=SIM" \
  -e "RUST_SGX_SDK=/sgx" \
  -e "INTEL_SGX_SDK=/opt/sgxsdk" \
  -w /code \
  baiduxlab/sgx-rust-experimental \
  bash
