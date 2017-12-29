#!/bin/bash

# Set default values if not already set as environment variables
: ${INTEL_SGX_SDK:="/opt/intel/sgxsdk"}
: ${RUST_SGX_SDK:="/sgx"}
: ${SGX_MODE:="SIM"}
: ${SGX_ARCH:="x64"}

SGX_COMMON_CFLAGS="-m64"
SGX_LIBRARY_PATH="$INTEL_SGX_SDK/lib64"
SGX_ENCLAVE_SIGNER="$INTEL_SGX_SDK/bin/x64/sgx_sign"
SGX_EDGER8R="$INTEL_SGX_SDK/bin/x64/sgx_edger8r"

if [ "$SGX_DEBUG" == "1" ]
then
	SGX_COMMON_CFLAGS="$SGX_COMMON_CFLAGS -O0 -g"
else
	SGX_COMMON_CFLAGS="$SGX_COMMON_CFLAGS -O2"
fi

# Variables
CWD=$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )
CC=gcc
RustEnclave_Include_Paths="-I$INTEL_SGX_SDK/include -I$INTEL_SGX_SDK/include/tlibc -I$INTEL_SGX_SDK/include/stlport -I$INTEL_SGX_SDK/include/epid -I ./enclave -I./include"
RustEnclave_Compile_Flags="$SGX_COMMON_CFLAGS -nostdinc -fvisibility=hidden -fpie -fstack-protector $RustEnclave_Include_Paths"

# Create target directory
echo $CWD
mkdir -p $CWD/target/enclave

# Create signed enclave object
#$(CC) $(RustEnclave_Compile_Flags) -c $CWD/libcontract_trusted/src/generated/trusted/Enclave_t.c -o $CWD/target/enclave/Enclave_t.o
