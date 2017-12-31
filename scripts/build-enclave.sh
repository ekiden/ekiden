#!/bin/bash

# TODO: Migrate this to cargo make?

### CONFIG ###
CWD=$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )

# Accept first parameter as CONTRACT (e.g. token)
if [ $# -eq 0 ]
then
  CONTRACT="token"
else
  CONTRACT=$1
fi
echo "Creating enclave for $CONTRACT"

# Set default values if not already set as environment variables
: ${BUILT_DIR:="$CWD/target/debug"}
: ${TARGET_DIR:="$CWD/target/enclave"}
: ${ENCLAVE_LDS:="$CWD/libcontract/utils/config/enclave.lds"}
: ${ENCLAVE_KEY:="$CWD/libcontract/keys/private.pem"}
: ${ENCLAVE_CONFIG:="$CWD/libcontract/utils/config/enclave.xml"}
: ${INTEL_SGX_SDK:="/opt/intel/sgxsdk"}
: ${RUST_SGX_SDK:="/sgx"}
: ${SGX_MODE:="SIM"}
: ${SGX_ARCH:="x64"}

# Commands
CC=g++
MAKE=make

### SGX SETTINGS ###
SGX_COMMON_CFLAGS="-m64"
SGX_LIBRARY_PATH="${INTEL_SGX_SDK}/lib64"
SGX_ENCLAVE_SIGNER="${INTEL_SGX_SDK}/bin/x64/sgx_sign"
SGX_EDGER8R="${INTEL_SGX_SDK}/bin/x64/sgx_edger8r"

if [ "$SGX_DEBUG" == "1" ]
then
  SGX_COMMON_CFLAGS="${SGX_COMMON_CFLAGS} -O0 -g"
else
  SGX_COMMON_CFLAGS="${SGX_COMMON_CFLAGS} -O2"
fi

if [ "$SGX_MODE" == "HW" ]
then
  Trts_Library_Name=sgx_trts
  Service_Library_Name=sgx_tservice
else
  Trts_Library_Name=sgx_trts_sim
  Service_Library_Name=sgx_tservice_sim
fi
Crypto_Library_Name=sgx_tcrypto
KeyExchange_Library_Name=sgx_tkey_exchange
ProtectedFs_Library_Name=sgx_tprotected_fs

# Create target directory
echo ${CWD}
mkdir -p ${CWD}/target/enclave

# Create signed enclave object
echo "Making compiler-rt/ in Rust SGX SDK"
${MAKE} -C ${RUST_SGX_SDK}/compiler-rt/ 2> /dev/null

echo "Building ${CONTRACT}"
cargo build -p ${CONTRACT}

# Link with correct libraries and use correct linker flags.
echo "Linking libraries"
${CC} ${SGX_COMMON_CFLAGS} \
  -Wl,--no-undefined -nostdlib -nodefaultlibs -nostartfiles \
  -L${SGX_LIBRARY_PATH} \
  -L${BUILT_DIR} \
  -L${RUST_SGX_SDK}/compiler-rt \
  -Wl,--whole-archive -l${Trts_Library_Name} -Wl,--no-whole-archive \
  -Wl,--start-group -lsgx_tstdc -lsgx_tstdcxx -l${Crypto_Library_Name} -lcompiler-rt-patch -l${CONTRACT} -Wl,--end-group \
  -Wl,-Bstatic -Wl,-Bsymbolic -Wl,--no-undefined \
  -Wl,-pie,-eenclave_entry -Wl,--export-dynamic \
  -Wl,--defsym,__ImageBase=0 \
  -Wl,--gc-sections \
  -Wl,--version-script=${ENCLAVE_LDS} \
  -o ${TARGET_DIR}/${CONTRACT}.so

echo "Signing enclave"
${SGX_ENCLAVE_SIGNER} sign \
  -key ${ENCLAVE_KEY} \
  -enclave ${TARGET_DIR}/${CONTRACT}.so \
  -out ${TARGET_DIR}/${CONTRACT}.signed.so \
  -config ${ENCLAVE_CONFIG}

echo "Signed enclave here:"
echo "${TARGET_DIR}/${CONTRACT}.signed.so"
echo "Done."
