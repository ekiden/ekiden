#!/bin/bash

PROJ_ROOT="$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )"
ias_spid=${IAS_SPID:-11111111111111111111111111111111}
ias_pkcs=${IAS_PKCS:-client.pfx}

if [ "$1" = "--client" ]; then
  contract="$2"
  mrenclave=$(python2 "$PROJ_ROOT/scripts/parse_enclave.py" "$PROJ_ROOT/target/enclave/$contract.signed.so" 2>/dev/null | grep ENCLAVEHASH | cut -f2)
  cargo run -p "$contract-client" -- --mr-enclave "$mr_enclave"
else
  contract="$1"
  cargo run -p compute "$PROJ_ROOT/target/enclave/$contract.signed.so" -- --ias-spid "$ias_spid" --ias-pkcs12 "$ias_pkcs"
fi
