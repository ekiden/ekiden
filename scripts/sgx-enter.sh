#!/bin/bash

EKIDEN="ekiden"
PROJ_ROOT=$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )
TENDERMINT_PORT=8880

ekiden_image=${EKIDEN_DOCKER_IMAGE:-ekiden/rust-sgx-sdk}
ekiden_shell=${EKIDEN_DOCKER_SHELL:-bash}
rust_sgx_sdk_dir=${RUST_SGX_SDK:-${PROJ_ROOT}/../rust-sgx-sdk}

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

if (( $# > 0 )); then
  if [ "$1" == "--bg" ]; then
    docker_exit_behavior="--detach"
  fi
  docker_args="export LD_LIBRARY_PATH=/opt/sgxsdk/lib64; export PATH=/root/.cargo/bin:"$PATH"; $@"
else
  docker_args="$ekiden_shell"
fi

# Start SGX Rust Docker container.
if [ ! "$(docker ps -q -f name=$EKIDEN)" ]; then
  if [ "$(docker ps -aq -f name=$EKIDEN)" ]; then
    docker start $EKIDEN
    docker exec -i -t $docker_exit_behavior $EKIDEN bash -c "$docker_args"
  else
    docker run -t -i \
      "$docker_exit_behavior" \
      --name "$EKIDEN" \
      -v ${PROJ_ROOT}:/code \
      -v ${PROJ_ROOT}/../deps:/deps \
      -v ${rust_sgx_sdk_dir}:/sgx \
      -e "SGX_MODE=SIM" \
      -e "RUST_SGX_SDK=/sgx" \
      -e "INTEL_SGX_SDK=/opt/sgxsdk" \
      -p "${TENDERMINT_PORT}:46657" \
      -w /code \
      "$ekiden_image" \
      bash -c "$docker_args"
  fi
else
  docker exec -i -t $docker_exit_behavior $EKIDEN bash -c "$docker_args"
fi
