#!/bin/bash

EKIDEN="ekiden"
CWD=$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )
TENDERMINT_PORT=8880

ekiden_image=${EKIDEN_DOCKER_IMAGE:-ekiden/rust-sgx-sdk}
ekiden_shell=${EKIDEN_DOCKER_SHELL:-bash}

which docker >/dev/null || {
  echo "ERROR: Please install Docker first."
  exit 1
}

# Start SGX Rust Docker container.
if [ ! "$(docker ps -q -f name=$EKIDEN)" ]; then
  if [ "$(docker ps -aq -f name=$EKIDEN)" ]; then
    docker start $EKIDEN
    docker exec -i -t $EKIDEN /usr/bin/env $ekiden_shell
  else
    docker run -t -i \
      --name "$EKIDEN" \
      -v ${CWD}:/code \
      -e "SGX_MODE=SIM" \
      -e "INTEL_SGX_SDK=/opt/sgxsdk" \
      -p "${TENDERMINT_PORT}:46657" \
      -w /code \
      "$ekiden_image" \
      /usr/bin/env $ekiden_shell
  fi
else
  docker exec -i -t $EKIDEN /usr/bin/env $ekiden_shell
fi
