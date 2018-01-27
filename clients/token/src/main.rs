#[macro_use]
extern crate clap;

#[macro_use]
extern crate client_utils;
#[macro_use]
extern crate compute_client;
extern crate libcontract_common;

#[macro_use]
extern crate token_api;

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

    println!("Creating");
    client.create(request).unwrap();

    println!("Transferring");
    client
        .transfer({
            let mut request = token::TransferRequest::new();
            request.set_sender("testaddr".to_string());
            request.set_destination("b".to_string());
            request.set_value(3);
            request
        })
        .unwrap();

    println!("Checking balances");
    let response = client
        .get_balance({
            let mut request = token::GetBalanceRequest::new();
            request.set_account("testaddr".to_string());
            request
        })
        .unwrap();
    assert_eq!(response.get_balance(), 7_999_999_999_999_999_997);
    let response = client
        .get_balance({
            let mut request = token::GetBalanceRequest::new();
            request.set_account("b".to_string());
            request
        })
        .unwrap();
    assert_eq!(response.get_balance(), 3);
    let response = client
        .get_balance({
            let mut request = token::GetBalanceRequest::new();
            request.set_account("poor".to_string());
            request
        })
        .unwrap();
    assert_eq!(response.get_balance(), 0);
}
