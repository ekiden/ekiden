#![feature(prelude_import)]
#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

extern crate protobuf;

extern crate libcontract_common;
#[macro_use]
extern crate libcontract_trusted;

#[macro_use]
extern crate token_api;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

mod token_contract;

use token_api::{CreateRequest, CreateResponse, TokenState, TransferRequest, TransferResponse, GetBalanceRequest, GetBalanceResponse};
use token_contract::TokenContract;

use libcontract_common::{with_contract_state, Address, Contract, ContractError};

create_enclave_api!();

fn create(request: &CreateRequest) -> Result<(TokenState, CreateResponse), ContractError> {
    let contract = TokenContract::new(
        &Address::from(request.get_sender().to_string()),
        request.get_initial_supply(),
        request.get_token_name().to_string(),
        request.get_token_symbol().to_string(),
    );

    let response = CreateResponse::new();

    Ok((contract.get_state(), response))
}

fn transfer(
    state: &TokenState,
    request: &TransferRequest,
) -> Result<(TokenState, TransferResponse), ContractError> {
    let state = with_contract_state(state, |contract: &mut TokenContract| {
        contract.transfer(
            &Address::from(request.get_sender().to_string()),
            &Address::from(request.get_destination().to_string()),
            request.get_value(),
        )?;

        Ok(())
    })?;

    let response = TransferResponse::new();

    Ok((state, response))
}

fn get_balance(
    state: &TokenState,
    request: &GetBalanceRequest,
) -> Result<GetBalanceResponse, ContractError> {
    let contract = TokenContract::from_state(state);
    let balance = contract.get_balance(&Address::from(request.get_account().to_string()))?;

    let mut response = GetBalanceResponse::new();
    response.set_balance(balance);

    Ok(response)
}
