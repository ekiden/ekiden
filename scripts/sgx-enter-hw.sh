#!/bin/bash -e

# Working directory is determined by using git, so we can use the same script
# with external repositories which use their own root.
WORK_DIR=$( git rev-parse --show-toplevel )
# Name of the ekiden container.
EKIDEN_CONTAINER_NAME=${EKIDEN_CONTAINER_NAME:-$(basename ${WORK_DIR})}

ekiden_image=${EKIDEN_DOCKER_IMAGE:-ekiden/rust-sgx-sdk}
ekiden_shell=${EKIDEN_DOCKER_SHELL:-bash}

which docker >/dev/null || {
  echo "ERROR: Please install Docker first."
  exit 1
}

# Start SGX Rust Docker container.
if [ ! "$(docker ps -q -f name=${EKIDEN_CONTAINER_NAME})" ]; then
  if [ "$(docker ps -aq -f name=${EKIDEN_CONTAINER_NAME})" ]; then
    docker start ${EKIDEN_CONTAINER_NAME}
    docker exec -i -t ${EKIDEN_CONTAINER_NAME} /usr/bin/env $ekiden_shell
  else
    # privileged for aesmd
    docker run -t -i \
      --privileged \
      --name "${EKIDEN_CONTAINER_NAME}" \
      -v ${WORK_DIR}:/code \
      -e "SGX_MODE=HW" \
      -e "INTEL_SGX_SDK=/opt/sgxsdk" \
      -w /code \
      "$ekiden_image" \
      /usr/bin/env $ekiden_shell
  fi
else
  docker exec -i -t ${EKIDEN_CONTAINER_NAME} /usr/bin/env $ekiden_shell
fi
