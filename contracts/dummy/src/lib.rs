#![feature(prelude_import)]
#![no_std]
extern crate sgx_tstd as std;
#[macro_use]
extern crate libcontract_trusted;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

// Create enclave glue.
create_enclave!();
