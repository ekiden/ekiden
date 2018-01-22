#![feature(prelude_import)]
#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate lazy_static;
extern crate protobuf;

extern crate libcontract_common;
#[macro_use]
extern crate libcontract_trusted;

#[macro_use]
extern crate key_manager_api;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

mod key_store;

use key_manager_api::{GetOrCreateKeyRequest, GetOrCreateKeyResponse};

use libcontract_common::ContractError;
use libcontract_trusted::dispatcher::Request;

use key_store::KeyStore;

create_enclave_api!();

fn get_or_create_key(
    request: &Request<GetOrCreateKeyRequest>,
) -> Result<GetOrCreateKeyResponse, ContractError> {
    let mut response = GetOrCreateKeyResponse::new();

    // Query the key store.
    {
        let mut key_store = KeyStore::get();
        response.set_key(key_store.get_or_create_key(
            // Unwrap here is safe as this contract requires mutual authentication.
            &request.get_client_mr_enclave().as_ref().unwrap(),
            request.get_name(),
            request.get_size() as usize,
        )?);
    }

    Ok(response)
}
