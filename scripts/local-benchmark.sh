./target/debug/consensus -x &
./target/debug/compute --disable-key-manager --port 9003 ./target/enclave/key-manager.signed.so &
./target/debug/compute ./target/enclave/token.signed.so &
