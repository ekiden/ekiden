extern crate libenclave_untrusted;

use libenclave_untrusted::enclave;
use libenclave_untrusted::rpc;

fn main() {
    // Create a new ekiden enclave from the given library.
    let simple = enclave::EkidenEnclave::new("enclave_dummy.signed.so").unwrap();

    // Fire off an RPC.
    let mut request = rpc::Request::new();
    request.set_method(String::from("hello_world"));
    let response = simple.call(&request).unwrap();
    println!("Response status={:?}", response.code);

    // Destroy the enclave.
    simple.destroy();
}
