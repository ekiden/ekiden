#[macro_use] extern crate clap;

#[macro_use] extern crate compute_client;
#[macro_use] extern crate client_utils;

#[macro_use] extern crate token_api;

use clap::{App, Arg};

create_client_api!();

fn main() {
    let mut client = contract_client!(token);

    // Create new token contract.
    let mut request = token::CreateRequest::new();
    request.set_sender("testaddr".to_string());
    request.set_token_name("Ekiden Token".to_string());
    request.set_token_symbol("EKI".to_string());
    request.set_initial_supply(8);

    let response = client.create(request).unwrap();

    println!("Response from contract: {:?}", response);

    let response = client.transfer({
        let mut request = token::TransferRequest::new();
        request.set_sender("testaddr".to_string());
        request.set_destination("b".to_string());
        request.set_value(3);
        request
    }).unwrap();

    println!("Response from contract: {:?}", response);
}
