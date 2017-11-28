#!/bin/bash

DATAPATH="/tmp"
GENESISPATH=$DATAPATH/genesis.json

# Check to see if docker is on the path
if [ ! $(which docker) ]; then
    echo "Please install docker"
    exit 1
fi

# Clear the data directory
if [ -f $GENESISPATH ]; then
    echo "Clearing Tendermint directory"
    docker run -it --rm -v "$DATAPATH:/tendermint" tendermint/tendermint unsafe_reset_all
else
    echo "Cannot recognize Tendermint directory"
fi

