#![feature(prelude_import)]
#![no_std]
#![feature(alloc)]

mod evm;

#[macro_use]
extern crate sgx_tstd as std;

extern crate libcontract_common;
#[macro_use]
extern crate libcontract_trusted;

#[macro_use]
extern crate ethtoken_api;

extern crate protobuf;

extern crate alloc;
extern crate bigint;
extern crate hexutil;
extern crate sha3;
extern crate sputnikvm;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

use ethtoken_api::{CreateTokenRequest, CreateTokenResponse, EthState, GetBalanceRequest,
                   GetBalanceResponse, InitStateRequest, InitStateResponse, TransferTokenRequest,
                   TransferTokenResponse};

use sputnikvm::{TransactionAction, ValidTransaction};

use libcontract_common::ContractError;

use bigint::{Address, Gas, H256, U256};
use hexutil::{read_hex, to_hex};
use sha3::{Digest, Keccak256};

use core::str::FromStr;
use std::rc::Rc;

use evm::fire_transactions_and_update_state;

create_enclave_api!();

fn create(
    state: &EthState,
    request: &CreateTokenRequest,
) -> Result<(EthState, CreateTokenResponse), ContractError> {
    println!("create creator={}", request.get_creator_address());

    let creator_addr = Address::from_str(request.get_creator_address()).unwrap();

    // EVM bytecode for ERC20 token contract (from https://ethereum.org/token) with the following parameters:
    //
    // decimals: 0
    // initialSupply: <filled from request>
    // tokenName: "Test"
    // tokenSymbol: "TST"
    //
    let mut bytecode: Vec<u8> = read_hex(include_str!("../resources/erc20.contract")).unwrap();
    // Add encoded initialSupply parameter.
    bytecode.extend_from_slice(&H256::from(request.get_initial_supply()));
    // Add remaining constructor parameters (tokenName, tokenSymbol).
    bytecode.extend_from_slice(&read_hex("0x000000000000000000000000000000000000000000000000000000000000006000000000000000000000000000000000000000000000000000000000000000a00000000000000000000000000000000000000000000000000000000000000004546573740000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000035453540000000000000000000000000000000000000000000000000000000000").unwrap());

    let transactions = [
        ValidTransaction {
            caller: Some(creator_addr),
            action: TransactionAction::Create,
            gas_price: Gas::zero(),
            gas_limit: Gas::max_value(),
            value: U256::zero(),
            input: Rc::new(bytecode),
            nonce: U256::zero(),
        },
    ];

    let (new_state, _) = fire_transactions_and_update_state(&transactions, state);

    // Compute address of new token contract. In practice, a web3 client handling a "create" action
    // returns a transaction hash, and the caller needs to wait until the next block is mined to
    // retrieve the contract's address. For simplicity, we manually compute the address and return
    // it immediately. The address is a function of the caller and nonce (see https://ethereum.stackexchange.com/questions/760/how-is-the-address-of-an-ethereum-contract-computed)
    //
    let token_contract_addr = {
        let mut vec = read_hex("0xd694").unwrap().to_vec();
        vec.extend_from_slice(&creator_addr);
        vec.extend_from_slice(&[0x80]);
        to_hex(&Keccak256::digest(&vec)[12..])
    };

    let mut response = CreateTokenResponse::new();
    response.set_contract_address(token_contract_addr.clone());

    Ok((new_state, response))
}

fn transfer(
    state: &EthState,
    request: &TransferTokenRequest,
) -> Result<(EthState, TransferTokenResponse), ContractError> {
    println!(
        "transfer amount={}, from={}, to={}",
        request.amount, request.from_address, request.to_address
    );

    let to_addr = Address::from_str(request.get_to_address()).unwrap();

    // Construct the EVM payload for this transaction.
    //
    // To call the contract's "transfer" method, we take the first 4 bytes from the Keccak256 hash
    // of the the function's signature, then append the parameters values (destination and amount),
    // encoded and padded according to the Ethereum ABI spec.
    //
    // For more information, see https://github.com/ethereum/wiki/wiki/Ethereum-Contract-ABI.
    //
    let mut payload =
        Keccak256::digest("transfer(address,uint256)".as_bytes()).as_slice()[..4].to_vec();
    payload.extend_from_slice(&H256::from(to_addr));
    payload.extend_from_slice(&H256::from(request.get_amount()));

    let caller = Address::from_str(request.get_from_address()).unwrap();
    let contract_addr = Address::from_str(request.get_contract_address()).unwrap();

    let transactions = [
        ValidTransaction {
            caller: Some(caller),
            action: TransactionAction::Call(contract_addr),
            gas_price: Gas::zero(),
            gas_limit: Gas::max_value(),
            value: U256::zero(),
            input: Rc::new(payload),
            nonce: U256::zero(),
        },
    ];

    let (new_state, _) = fire_transactions_and_update_state(&transactions, state);
    let response = TransferTokenResponse::new();

    Ok((new_state, response))
}

fn get_balance(
    state: &EthState,
    request: &GetBalanceRequest,
) -> Result<GetBalanceResponse, ContractError> {
    println!("get_balance addr={}", request.get_address());

    let address = Address::from_str(request.get_address()).unwrap();
    let contract_addr = Address::from_str(request.get_contract_address()).unwrap();

    // Construct the EVM payload for this transaction. See comment in transfer_tokens() for explanation.
    let mut payload = Keccak256::digest("balanceOf(address)".as_bytes()).as_slice()[..4].to_vec();
    payload.extend_from_slice(&H256::from(address));

    let transactions = [
        ValidTransaction {
            caller: Some(Address::default()),
            action: TransactionAction::Call(contract_addr),
            gas_price: Gas::zero(),
            gas_limit: Gas::max_value(),
            value: U256::zero(),
            input: Rc::new(payload),
            nonce: U256::zero(),
        },
    ];

    let (_, result) = fire_transactions_and_update_state(&transactions, state);

    let mut response = GetBalanceResponse::new();
    let result_as_u64 = U256::from(result.as_slice()).as_u64();
    response.set_balance(result_as_u64);

    Ok(response)
}

fn init_genesis_state(
    _request: &InitStateRequest,
) -> Result<(EthState, InitStateResponse), ContractError> {
    let state = EthState::new();
    let response = InitStateResponse::new();
    Ok((state, response))
}
