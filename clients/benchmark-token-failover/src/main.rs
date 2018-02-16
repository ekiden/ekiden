#[macro_use]
extern crate clap;
extern crate futures;
extern crate rand;
extern crate time;
extern crate tokio_core;

#[macro_use]
extern crate client_utils;
#[macro_use]
extern crate compute_client;
extern crate libcontract_common;

#[macro_use]
extern crate token_api;

use std::io::{self, Write};

use clap::{App, Arg};

use rand::{thread_rng, Rng};

use futures::future::Future;

create_client_api!();

/// Initializes the token scenario.
fn init<Backend>(client: &mut token::Client<Backend>, _runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    // Create new token contract.
    let mut request = token::CreateRequest::new();
    request.set_sender("bank".to_string());
    request.set_token_name("Ekiden Token".to_string());
    request.set_token_symbol("EKI".to_string());
    request.set_initial_supply(8);

    client.create(request).wait().unwrap();

    // Check balances.
    let response = client
        .get_balance({
            let mut request = token::GetBalanceRequest::new();
            request.set_account("bank".to_string());
            request
        })
        .wait()
        .unwrap();
    assert_eq!(response.get_balance(), 8_000_000_000_000_000_000);
}

/// Create a new random token address.
fn create_address() -> String {
    thread_rng().gen_ascii_chars().take(32).collect()
}

/// Runs the token failover scenario.
fn scenario<Backend>(client: &mut token::Client<Backend>)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    // Generate random addresses.
    let poor = create_address();

    let total_requests = 2000;
    let mut failures = 0;
    let mut times: Vec<u64> = vec![];

    for _ in 0..total_requests {
        io::stdout().flush().ok().unwrap();

        // Check balance.
        let start = time::precise_time_ns();
        let response = client
            .get_balance({
                let mut request = token::GetBalanceRequest::new();
                request.set_account(poor.clone());
                request
            })
            .wait();

        times.push((time::precise_time_ns() - start) / 1_000_000);

        match response {
            Ok(response) => {
                print!(".");
                assert_eq!(response.get_balance(), 0);
            }
            _ => {
                print!("!");
                failures += 1;
                continue;
            }
        }
    }

    println!("");
    println!("Failed requests: {} / {}", failures, total_requests);
    println!("Request timings:\n{:?}", times);
}

/// Finalize the token failover scenario.
fn finalize<Backend>(_client: &mut token::Client<Backend>, _runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
}

fn main() {
    let mut client = contract_client!(token);
    init(&mut client, 1, 1);
    scenario(&mut client);
    finalize(&mut client, 1, 1);
}
