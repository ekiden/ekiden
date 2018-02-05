#[macro_use]
extern crate clap;

#[macro_use]
extern crate client_utils;
#[macro_use]
extern crate compute_client;
extern crate libcontract_common;

#[macro_use]
extern crate ethtoken_api;

use clap::{App, Arg};

create_client_api!();

// Initial supply of tokens.
const INITIAL_SUPPLY: u64 = 1_000_000;

// Address of token creator. Can be anything but must parse to a valid Ethereum address (160-bit).
const TOKEN_CREATOR: &str = "0x4e4f41484e4f41484e4f41484e4f41484e4f4148";

// Amount to transfer from this client.
const TRANSFER_AMOUNT: u64 = 3;

// Address to transfer tokens to.
const TRANSFER_TO_ADDR: &str = "0x57415252454e57415252454e57415252454e0000";

// Address of created contract (set by init method).
static mut CONTRACT_ADDR: Option<String> = None;

/// Initializes the ethtoken scenario.
fn init<Backend>(client: &mut ethtoken::Client<Backend>, _runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    // Initialize empty state.
    client
        .init_genesis_state(ethtoken::InitStateRequest::new())
        .unwrap();

    // Create new ERC20 token contract. Returns the address of the newly created contract.
    // When instantiated, the contract automatically assigns all initial tokens to the contract's
    // creator (i.e. the caller). The token name and symbol are hardcoded in the contract bytecode
    // so they aren't specified here.
    println!(
        "Creating token contract with {} initial tokens (creator address {})",
        INITIAL_SUPPLY, TOKEN_CREATOR
    );
    let contract_addr = client
        .create({
            let mut req = ethtoken::CreateTokenRequest::new();
            req.set_creator_address(TOKEN_CREATOR.to_string());
            req.set_initial_supply(INITIAL_SUPPLY);
            req
        })
        .unwrap()
        .get_contract_address()
        .to_string();

    unsafe {
        CONTRACT_ADDR = Some(contract_addr.clone());
    }
    println!("Token contract address: {}", contract_addr);

    // Request the token balance of the creator's address. Should equal the initial supply.
    let balance = client
        .get_balance({
            let mut req = ethtoken::GetBalanceRequest::new();
            req.set_contract_address(contract_addr.clone());
            req.set_address(TOKEN_CREATOR.to_string());
            req
        })
        .unwrap()
        .get_balance();

    println!("\nBalance of address {} = {}", TOKEN_CREATOR, balance);
    assert_eq!(
        balance, INITIAL_SUPPLY,
        "Creator did not receive initial tokens"
    );
}

/// Runs the ethtoken scenario.
fn scenario<Backend>(client: &mut ethtoken::Client<Backend>)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    // Transfer tokens from the creator to a given address.
    println!(
        "Transferring {} tokens from {} to {}",
        TRANSFER_AMOUNT, TOKEN_CREATOR, TRANSFER_TO_ADDR
    );

    client
        .transfer({
            let mut req = ethtoken::TransferTokenRequest::new();
            unsafe {
                req.set_contract_address(CONTRACT_ADDR.as_ref().unwrap().clone());
            }
            req.set_from_address(TOKEN_CREATOR.to_string());
            req.set_to_address(TRANSFER_TO_ADDR.to_string());
            req.set_amount(TRANSFER_AMOUNT);
            req
        })
        .unwrap();
}

/// Finalize the ethtoken scenario.
fn finalize<Backend>(client: &mut ethtoken::Client<Backend>, runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    // Check the new balance of the creator's address.
    let creator_balance = client
        .get_balance({
            let mut req = ethtoken::GetBalanceRequest::new();
            unsafe {
                req.set_contract_address(CONTRACT_ADDR.as_ref().unwrap().clone());
            }
            req.set_address(TOKEN_CREATOR.to_string());
            req
        })
        .unwrap()
        .get_balance();

    println!(
        "\nBalance of address {} = {}",
        TOKEN_CREATOR, creator_balance
    );
    assert_eq!(
        creator_balance,
        INITIAL_SUPPLY - TRANSFER_AMOUNT * runs as u64,
        "Tokens not debited from sender"
    );

    // Check the balance for the destination address.
    let dest_balance = client
        .get_balance({
            let mut req = ethtoken::GetBalanceRequest::new();
            unsafe {
                req.set_contract_address(CONTRACT_ADDR.as_ref().unwrap().clone());
            }
            req.set_address(TRANSFER_TO_ADDR.to_string());
            req
        })
        .unwrap()
        .get_balance();

    println!("Balance of address {} = {}", TRANSFER_TO_ADDR, dest_balance);
    assert_eq!(
        dest_balance,
        TRANSFER_AMOUNT * runs as u64,
        "Tokens not transferred"
    );
}

#[cfg(feature = "benchmark")]
fn main() {
    let results = benchmark_client!(ethtoken, init, scenario, finalize);
    results.show();
}

#[cfg(not(feature = "benchmark"))]
fn main() {
    let mut client = contract_client!(ethtoken);
    init(&mut client, 1, 1);
    scenario(&mut client);
    finalize(&mut client, 1, 1);
}