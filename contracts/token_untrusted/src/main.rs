extern crate sgx_types;
extern crate sgx_urts;
extern crate protobuf;

mod enclave_rpc;
mod enclave;
mod errors;

fn main() {
    // Create a new ekiden enclave from the given library.
    let simple = enclave::EkidenEnclave::new("enclave.signed.so").unwrap();

    // Fire off an RPC.
    let mut request = enclave_rpc::Request::new();
    request.set_method(String::from("hello_world"));
    let response = simple.call(&request).unwrap();
    println!("Response status={:?}", response.code);

    // Destroy the enclave.
    simple.destroy();
}
