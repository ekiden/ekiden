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

fn main() {
    let mut client = contract_client!(ethtoken);
    client.init_genesis_state(ethtoken::InitStateRequest::new()).unwrap();

    // Address of token creator. Can be anything but must parse to a valid Ethereum address (160-bit).
    let token_creator = "0x4e4f41484e4f41484e4f41484e4f41484e4f4148";

    // Initial supply of tokens.
    let initial_supply = 1000;

    // Create new ERC20 token contract. Returns the address of the newly created contract.
    // When instantiated, the contract automatically assigns all initial tokens to the contract's
    // creator (i.e. the caller). The token name and symbol are hardcoded in the contract bytecode
    // so they aren't specified here.
    println!("Creating token contract with {} initial tokens (creator address {})", initial_supply, token_creator);
    let contract_addr = client
        .create_token({
            let mut req = ethtoken::CreateTokenRequest::new();
            req.set_creator_address(token_creator.to_string());
            req.set_initial_supply(initial_supply);
            req
        })
        .unwrap()
        .get_contract_address().to_string();

    println!("Token contract address: {}", contract_addr);

    // Request the token balance of the creator's address. Should equal the initial supply.
    let balance = client
        .get_balance({
            let mut req = ethtoken::GetBalanceRequest::new();
            req.set_contract_address(contract_addr.clone());
            req.set_address(token_creator.to_string());
            req
        })
        .unwrap()
        .get_balance();

    println!("\nBalance of address {} = {}", token_creator, balance);
    assert_eq!(balance, initial_supply, "Creator did not receive initial tokens");

    // Transfer tokens from the creator to a given address.
    let transfer_amount = 100;
    let transfer_to_addr = "0x57415252454e57415252454e57415252454e0000";
    println!("Transferring {} tokens from {} to {}", transfer_amount, token_creator, transfer_to_addr);
    let _res = client
        .transfer_tokens({
            let mut req = ethtoken::TransferTokenRequest::new();
            req.set_contract_address(contract_addr.clone());
            req.set_from_address(token_creator.to_string());
            req.set_to_address(transfer_to_addr.to_string());
            req.set_amount(transfer_amount);
            req
        })
        .unwrap();

    // Check the new balance of the creator's address.
    let creator_balance = client
        .get_balance({
            let mut req = ethtoken::GetBalanceRequest::new();
            req.set_contract_address(contract_addr.clone());
            req.set_address(token_creator.to_string());
            req
        })
        .unwrap()
        .get_balance();

    println!("\nBalance of address {} = {}", token_creator, creator_balance);
    assert_eq!(creator_balance, initial_supply-transfer_amount, "Tokens not debited from sender");

    // Check the balance for the destination address.
    let dest_balance = client
        .get_balance({
            let mut req = ethtoken::GetBalanceRequest::new();
            req.set_contract_address(contract_addr.clone());
            req.set_address(transfer_to_addr.to_string());
            req
        })
        .unwrap()
        .get_balance();

    println!("Balance of address {} = {}", transfer_to_addr, dest_balance);
    assert_eq!(dest_balance, transfer_amount, "Tokens not transferred");

    println!("\nSuccess!");
}
