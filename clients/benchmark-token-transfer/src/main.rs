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
const ACCOUNT_DST: &str = "dest";

/// Initializes the token transfer scenario.
fn init<Backend>(client: &mut token::Client<Backend>, runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    // TODO: Automatically choose an initial supply to accomodate larger runs.
    assert!(runs <= 1_000_000_000_000_000_000);

    // Create new token contract.
    let mut request = token::CreateRequest::new();
    request.set_sender(ACCOUNT_BANK.to_owned());
    request.set_token_name("Ekiden Token".to_owned());
    request.set_token_symbol("EKI".to_owned());
    request.set_initial_supply(1);

    client.create(request).unwrap();
}

/// Runs the token transfer scenario.
fn scenario<Backend>(client: &mut token::Client<Backend>)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    // Transfer some funds.
    client
        .transfer({
            let mut request = token::TransferRequest::new();
            request.set_sender(ACCOUNT_BANK.to_owned());
            request.set_destination(ACCOUNT_DST.to_owned());
            request.set_value(1);
            request
        })
        .unwrap();
}

/// Finalize the token transfer scenario.
fn finalize<Backend>(client: &mut token::Client<Backend>, runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    // Check final balance.
    let response = client
        .get_balance({
            let mut request = token::GetBalanceRequest::new();
            request.set_account(ACCOUNT_DST.to_owned());
            request
        })
        .unwrap();
    assert_eq!(response.get_balance(), runs as u64);
}

fn main() {
    let results = benchmark_client!(token, init, scenario, finalize);
    results.show();
}
