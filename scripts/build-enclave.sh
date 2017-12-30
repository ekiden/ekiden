#!/bin/bash

### CONFIG ###
CWD=$( cd "$( dirname "${BASH_SOURCE[0]}" )/.." && pwd )

# Accept first parameter as CONTRACT_PATH
if [ $# -eq 0 ]
then
  CONTRACT_PATH="$CWD/target/debug/libtoken.a"
else
  CONTRACT_PATH=$1
fi
echo "Creating enclave from $CONTRACT_PATH"

# Set default values if not already set as environment variables
: ${ENCLAVE_T_DIR:="$CWD/libcontract_trusted/src/generated/trusted"}
: ${TARGET_DIR:="$CWD/target/enclave"}
: ${ENCLAVE_LDS:="$CWD/Enclave.lds"}
: ${ENCLAVE_KEY:="$CWD/Enclave_private.pem"}
: ${ENCLAVE_CONFIG:="$CWD/Enclave.config.xml"}
: ${INTEL_SGX_SDK:="/opt/intel/sgxsdk"}
: ${RUST_SGX_SDK:="/sgx"}
: ${SGX_MODE:="SIM"}
: ${SGX_ARCH:="x64"}

# Commands
CC=gcc
MAKE=make

### SGX SETTINGS ###
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

### Compile flags
RustEnclave_Include_Paths="-I$INTEL_SGX_SDK/include -I$INTEL_SGX_SDK/include/tlibc -I$INTEL_SGX_SDK/include/stlport -I$INTEL_SGX_SDK/include/epid -I$ENCLAVE_T_DIR"
RustEnclave_Compile_Flags="$SGX_COMMON_CFLAGS -nostdinc -fvisibility=hidden -fpie -fstack-protector $RustEnclave_Include_Paths"
RustEnclave_Link_Libs="-L$TARGET_DIR -lcompiler-rt-patch -lenclave"
RustEnclave_Link_Flags="$SGX_COMMON_CFLAGS -Wl,--no-undefined \
  -nostdlib -nodefaultlibs -nostartfiles -L$SGX_LIBRARY_PATH \
  -Wl,--whole-archive -l$Trts_Library_Name -Wl,--no-whole-archive \
  -Wl,--start-group -lsgx_tstdc -lsgx_tstdcxx -l$Crypto_Library_Name $RustEnclave_Link_Libs -Wl,--end-group \
  -Wl,-Bstatic -Wl,-Bsymbolic -Wl,--no-undefined \
  -Wl,-pie,-eenclave_entry -Wl,--export-dynamic  \
  -Wl,--defsym,__ImageBase=0 \
  -Wl,--gc-sections \
  -Wl,--version-script=$ENCLAVE_LDS"

# Create target directory
echo $CWD
mkdir -p $CWD/target/enclave

# Create signed enclave object
echo "Making compiler-rt/ in Rust SGX SDK"
$MAKE -C $RUST_SGX_SDK/compiler-rt/ 2> /dev/null
cp $RUST_SGX_SDK/compiler-rt/libcompiler-rt-patch.a $TARGET_DIR/
echo "Compiling Enclave_t.o"
$CC $RustEnclave_Compile_Flags -c $ENCLAVE_T_DIR/Enclave_t.c -o $TARGET_DIR/Enclave_t.o
cp $CONTRACT_PATH $TARGET_DIR/libenclave.a
echo "Compiling enclave.so"
$CC $TARGET_DIR/Enclave_t.o -o $TARGET_DIR/enclave.so $RustEnclave_Link_Flags
echo "Signing enclave"
$SGX_ENCLAVE_SIGNER sign -key $ENCLAVE_KEY -enclave $TARGET_DIR/enclave.so -out $TARGET_DIR/enclave.signed.so -config $ENCLAVE_CONFIG
echo "Signed enclave here:"
echo "$TARGET_DIR/enclave.signed.so"
echo "Done."
