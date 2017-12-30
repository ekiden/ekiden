#!/bin/bash -e

# TODO: Migrate this to cargo make?

work_dir=$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )
SGX_RUST_SDK=/sgx

enclave=$1

# Ensure compiler-rt-patch is built.
make -C ${SGX_RUST_SDK}/compiler-rt/ 2> /dev/null

# Build enclave library.
cargo build --release -p ${enclave}

# Link with correct libraries and use correct linker flags.
g++ -m64 -O0 -g -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles \
    -L${work_dir}/target/release \
    -L${SGX_SDK}/lib64 \
    -L${SGX_RUST_SDK}/compiler-rt \
    -Wl,--whole-archive -lsgx_trts_sim -Wl,--no-whole-archive \
    -Wl,--start-group -lsgx_tstdc -lsgx_tstdcxx -lsgx_tcrypto -lcompiler-rt-patch -l${enclave} -Wl,--end-group \
    -Wl,-Bstatic -Wl,-Bsymbolic -Wl,--no-undefined \
    -Wl,-pie,-eenclave_entry -Wl,--export-dynamic \
    -Wl,--defsym,__ImageBase=0 \
    -Wl,--gc-sections \
    -Wl,--version-script=${work_dir}/libenclave/utils/config/enclave.lds \
    -o ${work_dir}/target/release/${enclave}.so

# Sign enclave.
${SGX_SDK}/bin/x64/sgx_sign sign \
    -key ${work_dir}/libenclave/keys/private.pem \
    -enclave ${work_dir}/target/release/${enclave}.so \
    -out ${work_dir}/target/release/${enclave}.signed.so \
    -config ${work_dir}/libenclave/utils/config/enclave.xml
