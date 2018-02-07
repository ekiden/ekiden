#!/bin/bash

PROJ_ROOT=$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )
SCRIPTS="$PROJ_ROOT/scripts"

docker rm -f tendermint >/dev/null 2>&1
bash "$SCRIPTS/tendermint-clear.sh" > /dev/null
bash "$SCRIPTS/tendermint-start.sh" --bg > /dev/null
bash "$SCRIPTS/sgx-enter.sh" --bg 'ps -C compute -o pid --no-headers | xargs kill 2>/dev/null'
bash "$SCRIPTS/sgx-enter.sh" --bg /root/.cargo/bin/cargo run -p compute consensus
bash "$SCRIPTS/sgx-enter.sh" --bg bash /code/scripts/run_contract.sh key-manager -p 9003 --disable-key-manager --consensus-host disabled
bash "$SCRIPTS/sgx-enter.sh" bash /code/scripts/run_contract.sh "$1" 2>&1
