#!/bin/bash

CWD=$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )
DATA_PATH="/tmp/tendermint"
GENESIS_PATH=${DATA_PATH}/genesis.json

# Check to see if docker is on the path
if [ ! $(which docker) ]; then
  echo "Please install docker"
  exit 1
fi

# Initialize the data directory
if [ -f $GENESIS_PATH ]; then echo "Tendermint directory already initialized"
else
  echo "Initializing Tendermint data directory"
  docker run -it --rm -v "${DATA_PATH}:/tendermint" tendermint/tendermint init
fi

# Start
docker run -it --rm \
  --name "tendermint" \
  --network container:storage \
  -v "${DATA_PATH}:/tendermint" \
  tendermint/tendermint node --consensus.create_empty_blocks=false
  #--net=host 
  #--proxy_app=dummy

