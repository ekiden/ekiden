#!/bin/sh -eu
TM_VERSION=$1
IMAGE_TAG=tendermint/tendermint:$TM_VERSION
VALIDATOR_POWER=10

cat >genesis.json <<EOF
{
  "genesis_time": "2018-03-03T00:00:00.000Z",
  "chain_id": "ekidentm-test",
  "validators": [],
  "app_hash": ""
}
EOF

for n in val1 val2 val3; do
  # mkdir "validators/$n"
  # docker run "$IMAGE_TAG" gen_validator > "validators/$n/priv_validator.json"
  jq ".pub_key as \$k | {pub_key: \$k, power: $VALIDATOR_POWER, name: \"$n\"}" <"validators/$n/priv_validator.json" >"validators/$n/pub_validator.json"
  jq ".validators |= .+ [$(cat validators/$n/pub_validator.json)]" <genesis.json >tmpgenesis
  mv tmpgenesis genesis.json
done

for n in val1 val2 val3; do
  scp -F ./ssh_config genesis.json "$n:~/.tendermint" &
  scp -F ./ssh_config "validators/$n/priv_validator.json" "$n:~/.tendermint/config" &
done
wait
