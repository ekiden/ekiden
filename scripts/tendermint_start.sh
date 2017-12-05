#!/bin/bash

DATAPATH="/tmp"
GENESISPATH=$DATAPATH/genesis.json
HOSTPORT=8880

# Check to see if docker is on the path
if [ ! $(which docker) ]; then
    echo "Please install docker"
    exit 1
fi

# Initialize the data directory
if [ -f $GENESISPATH ]; then echo "Tendermint directory already initialized"
else
    echo "Initializing Tendermint data directory"
    docker run -it --rm -v "$DATAPATH:/tendermint" tendermint/tendermint init
fi

# Start
docker run -it --rm -v "$DATAPATH:/tendermint" --net=host -p "$HOSTPORT:46657" tendermint/tendermint node
# 
#--proxy_app=dummy

