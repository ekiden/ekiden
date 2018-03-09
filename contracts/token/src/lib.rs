#![feature(use_extern_macros)]

extern crate protobuf;

extern crate ekiden_core_common;
extern crate ekiden_core_trusted;

#[macro_use]
extern crate token_api;

mod token_contract;

use protobuf::Message;

use token_api::{with_api, CreateRequest, CreateResponse, GetBalanceRequest, GetBalanceResponse,
                TransferRequest, TransferResponse};
use token_contract::TokenContract;

use ekiden_core_common::Result;
use ekiden_core_common::contract::{with_contract_state, Address, Contract};
use ekiden_core_trusted::db::{Database, DatabaseHandle};
use ekiden_core_trusted::rpc::create_enclave_rpc;

// Create enclave RPC handlers.
with_api! {
    create_enclave_rpc!(api);
}

fn create(request: &CreateRequest) -> Result<CreateResponse> {
    let mut db = DatabaseHandle::instance();
    let contract = TokenContract::new(
        &Address::from(request.get_sender().to_string()),
        request.get_initial_supply(),
        request.get_token_name().to_string(),
        request.get_token_symbol().to_string(),
    );

    db.insert(b"state", &contract.get_state().write_to_bytes()?);

    Ok(CreateResponse::new())
}

fn transfer(request: &TransferRequest) -> Result<TransferResponse> {
    let mut db = DatabaseHandle::instance();
    let state = protobuf::parse_from_bytes(&db.get(b"state").unwrap())?;

    let state = with_contract_state(&state, |contract: &mut TokenContract| {
        contract.transfer(
            &Address::from(request.get_sender().to_string()),
            &Address::from(request.get_destination().to_string()),
            request.get_value(),
        )?;

        Ok(())
    })?;

    db.insert(b"state", &state.write_to_bytes()?);

    Ok(TransferResponse::new())
}

fn get_balance(request: &GetBalanceRequest) -> Result<GetBalanceResponse> {
    let db = DatabaseHandle::instance();
    let contract =
        TokenContract::from_state(&protobuf::parse_from_bytes(&db.get(b"state").unwrap())?);
    let balance = contract.get_balance(&Address::from(request.get_account().to_string()))?;

    let mut response = GetBalanceResponse::new();
    response.set_balance(balance);

    Ok(response)
}
