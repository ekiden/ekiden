#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate rand;

#[macro_use]
extern crate client_utils;
#[macro_use]
extern crate compute_client;
extern crate libcontract_common;

#[macro_use]
extern crate token_api;

use clap::{App, Arg};

use rand::{thread_rng, Rng};

create_client_api!();

const ACCOUNT_BANK: &str = "bank";
const ACCOUNT_COUNT: usize = 1000;

lazy_static! {
    static ref ACCOUNTS: Vec<String> = {
        // Generate some random account names.
        let mut accounts = vec![];

        for _ in 0..ACCOUNT_COUNT {
            accounts.push(thread_rng().gen_ascii_chars().take(32).collect());
        }

        accounts
    };
}

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
    // Choose a random account.
    let destination = thread_rng().choose(&ACCOUNTS).unwrap();

    // Transfer some funds.
    client
        .transfer({
            let mut request = token::TransferRequest::new();
            request.set_sender(ACCOUNT_BANK.to_owned());
            request.set_destination(destination.clone());
            request.set_value(1);
            request
        })
        .unwrap();
}

/// Finalize the token transfer scenario.
fn finalize<Backend>(_client: &mut token::Client<Backend>, _runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
}

fn main() {
    let results = benchmark_client!(token, init, scenario, finalize);
    results.show();
}
