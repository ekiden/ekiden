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

const ACCOUNT_BANK: &str = "bank";

/// Initializes the token get balance scenario.
fn init<Backend>(client: &mut token::Client<Backend>, _runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    // Create new token contract.
    let mut request = token::CreateRequest::new();
    request.set_sender(ACCOUNT_BANK.to_owned());
    request.set_token_name("Ekiden Token".to_owned());
    request.set_token_symbol("EKI".to_owned());
    request.set_initial_supply(1);

    client.create(request).unwrap();
}

/// Runs the token get balance scenario.
fn scenario<Backend>(client: &mut token::Client<Backend>)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    // Check balance.
    let response = client
        .get_balance({
            let mut request = token::GetBalanceRequest::new();
            request.set_account(ACCOUNT_BANK.to_owned());
            request
        })
        .unwrap();
    assert_eq!(response.get_balance(), 1_000_000_000_000_000_000);
}

/// Finalize the token get balance scenario.
fn finalize<Backend>(_client: &mut token::Client<Backend>, _runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    // No action needed.
}

fn main() {
    let results = benchmark_client!(token, init, scenario, finalize);
    results.show();
}
