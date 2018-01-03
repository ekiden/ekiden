extern crate libcontract_untrusted;
extern crate protobuf;

use std::env;
use libcontract_untrusted::enclave;

mod generated;

use generated::token_state::{CreateRequest, CreateResponse, TransferRequest, TransferResponse};

fn main() {
    let enclave_filename = env::args().nth(1).expect("Usage: compute enclave_filename");

    // Create a new ekiden enclave from the given library.
    let e = enclave::EkidenEnclave::new(&enclave_filename).unwrap();

    // Create new token contract.
    let mut request = CreateRequest::new();
    request.set_sender("testaddr".to_string());
    request.set_token_name("Ekiden Token".to_string());
    request.set_token_symbol("EKI".to_string());
    request.set_initial_supply(8);

    let response: CreateResponse = e.call("create", &request).unwrap();

    println!("State after create:\n{:?}", response.get_state());

    // Transfer some tokens.
    let mut request = TransferRequest::new();
    request.set_state(response.get_state().clone());
    request.set_sender("testaddr".to_string());
    request.set_destination("anotheraddr".to_string());
    request.set_value(1000);

    let response: TransferResponse = e.call("transfer", &request).unwrap();

    println!("State after transfer:\n{:?}", response.get_state());

    // Destroy the enclave.
    e.destroy();
}
