#![feature(use_extern_macros)]
#![feature(prelude_import)]
#![no_std]

#[macro_use]
extern crate sgx_tstd as std;

#[macro_use]
extern crate lazy_static;
extern crate protobuf;

extern crate ekiden_core_common;
extern crate ekiden_core_trusted;

extern crate key_manager_api;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

mod key_store;

use ekiden_core_common::Result;
use ekiden_core_trusted::rpc::create_enclave_rpc;
use ekiden_core_trusted::rpc::request::Request;

use key_manager_api::{with_api, GetOrCreateKeyRequest, GetOrCreateKeyResponse};

use key_store::KeyStore;

// Create enclave RPC handlers.
with_api! {
    create_enclave_rpc!(api);
}

fn get_or_create_key(request: &Request<GetOrCreateKeyRequest>) -> Result<GetOrCreateKeyResponse> {
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
