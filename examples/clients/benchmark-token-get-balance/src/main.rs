#[macro_use]
extern crate clap;
extern crate futures;
#[macro_use]
extern crate lazy_static;
extern crate rand;
extern crate tokio_core;

#[macro_use]
extern crate client_utils;
#[macro_use]
extern crate compute_client;
extern crate libcontract_common;

#[macro_use]
extern crate token_api;

use clap::{App, Arg};
use futures::Future;

use rand::{thread_rng, Rng};

create_client_api!();

const ACCOUNT_BANK: &str = "bank";

const OTHER_ACCOUNT_COUNT: usize = 200;
lazy_static! {
    static ref OTHER_ACCOUNTS: Vec<String> = {
        // Generate some random account names.
        let mut accounts = vec![];

        for _ in 0..OTHER_ACCOUNT_COUNT {
            accounts.push(thread_rng().gen_ascii_chars().take(32).collect());
        }

        accounts
    };
}

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

    client.create(request).wait().unwrap();

    // Populate the other accounts.
    for other_account in OTHER_ACCOUNTS.iter() {
        client
            .transfer({
                let mut request = token::TransferRequest::new();
                request.set_sender(ACCOUNT_BANK.to_owned());
                request.set_destination(other_account.clone());
                request.set_value(1);
                request
            })
            .wait()
            .unwrap();
    }
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
        .wait()
        .unwrap();
    assert_eq!(
        response.get_balance(),
        1_000_000_000_000_000_000 - OTHER_ACCOUNT_COUNT as u64
    );
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
