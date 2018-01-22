#![cfg_attr(feature = "sgx", no_std)]
#![cfg_attr(feature = "sgx", feature(prelude_import))]

extern crate protobuf;

#[cfg(feature = "sgx")]
extern crate sgx_tstd as std;

#[macro_use]
extern crate libcontract_common;

#[cfg_attr(feature = "sgx", allow(unused))]
#[cfg_attr(feature = "sgx", prelude_import)]
#[cfg(feature = "sgx")]
use std::prelude::v1::*;

#[macro_use]
mod api;
mod generated;

pub use generated::api::*;

#[macro_export]
macro_rules! unpack_vals {
    ($features:expr, ($($ks:ident),+), $dofn:block) => {
        match ($(unpack_val!($features, $ks),)+) {
            ($(Some($ks),)+) => $dofn,
            _ => None
        }
    };
    ($features:expr, ($($ks:ident),+)) => {
        unpack_vals!($features, ($($ks),+), { Some(($($ks,)+)) })
    }
}

#[macro_export]
macro_rules! unpack_val {
    ($features:expr, $k:ident) => {
        $features.get(stringify!($k)).map(|v| {
            v.get_float_list().get_value().first().map(|&v| v as f64).unwrap_or(0f64)
        })
    }
}
