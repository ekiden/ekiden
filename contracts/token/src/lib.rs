#![feature(prelude_import)]
#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

extern crate protobuf;

#[macro_use]
extern crate libcontract_trusted;
extern crate libcontract_common;

#[macro_use]
extern crate token_api;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

mod token_contract;

use token_contract::TokenContract;
use token_api::{TokenState, TransferRequest, TransferResponse, CreateRequest, CreateResponse};

use libcontract_common::{Address, Contract, ContractError, with_contract_state};

create_enclave_api!();

fn create(_: TokenState, request: CreateRequest) -> Result<(TokenState, CreateResponse), ContractError> {
    let contract = TokenContract::new(
        &Address::from(request.get_sender().to_string()),
        request.get_initial_supply(),
        request.get_token_name().to_string(),
        request.get_token_symbol().to_string()
    );

    let mut response = CreateResponse::new();

    Ok((contract.get_state(), response))
}

fn transfer(state: TokenState, request: TransferRequest) -> Result<(TokenState, TransferResponse), ContractError> {
    let state = with_contract_state(&state, |contract: &mut TokenContract| {
        contract.transfer(
            &Address::from(request.get_sender().to_string()),
            &Address::from(request.get_destination().to_string()),
            request.get_value()
        )?;

        Ok(())
    })?;

    let mut response = TransferResponse::new();

    Ok((state, response))
}
