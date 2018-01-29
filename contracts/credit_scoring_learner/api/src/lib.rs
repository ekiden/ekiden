#![cfg_attr(feature = "sgx", no_std)]
#![cfg_attr(feature = "sgx", feature(prelude_import))]

extern crate protobuf;

#[cfg(feature = "sgx")]
extern crate sgx_tstd as std;

extern crate learner_api;

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
pub use learner_api::{CreateRequest, CreateResponse, Examples, InferenceRequest,
                      InferenceResponse, LearnerState, TrainingRequest, TrainingResponse};
