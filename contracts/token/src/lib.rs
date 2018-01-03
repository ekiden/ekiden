#![feature(prelude_import)]
#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate libcontract_trusted;
extern crate protobuf;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

mod token_contract;
mod generated;

use protobuf::{Message, MessageStatic};

use token_contract::TokenContract;
use generated::token_state::{TransferRequest, TransferResponse};
use libcontract_trusted::common::address::Address;
use libcontract_trusted::common::contract::{Contract, with_contract_state};
use libcontract_trusted::common::contract_error::ContractError;

// Create enclave.
create_enclave! {
    rpc transfer(TransferRequest) -> TransferResponse;
}

fn transfer(request: TransferRequest) -> Result<TransferResponse, ContractError> {
    let state = with_contract_state(request.get_state(), |contract: &mut TokenContract| {
        contract.transfer(
            &Address::from(request.get_sender().to_string()),
            &Address::from(request.get_destination().to_string()),
            request.get_value()
        )?;

        Ok(())
    })?;

    let mut response = TransferResponse::new();
    response.set_state(state);

    Ok(response)
}
