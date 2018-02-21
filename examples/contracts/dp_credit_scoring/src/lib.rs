#![feature(prelude_import)]
#![no_std]

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

extern crate sgx_rand as rand;
#[macro_use]
extern crate sgx_tstd as std;

extern crate ndarray;
extern crate ndarray_rand;
extern crate protobuf;
extern crate serde;
extern crate serde_cbor;
#[macro_use]
extern crate serde_derive;

extern crate libcontract_common;
#[macro_use]
extern crate libcontract_trusted;

#[macro_use]
extern crate dp_credit_scoring_api as api;

mod contract;
mod dpml;
#[macro_use]
mod macros;

use api::*;
use contract::Learner;
use libcontract_common::{Address, Contract, ContractError};
use ndarray::{Array, Array2};
#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

create_enclave_api!();

fn create(req: &CreateRequest) -> Result<(LearnerState, CreateResponse), ContractError> {
    let learner = Learner::new(Address::from(req.get_requester().to_string()));
    Ok((learner.get_state(), CreateResponse::new()))
}

fn train(
    state: &LearnerState,
    req: &TrainingRequest,
) -> Result<(LearnerState, TrainingResponse), ContractError> {
    let mut learner = check_owner!(state, req);

    let inputs = req.get_inputs();
    let xs = Array::from_shape_vec(
        (inputs.get_rows() as usize, inputs.get_cols() as usize),
        inputs.get_data().iter().map(|&v| v as f64).collect(),
    ).unwrap();
    let targets = req.get_targets();
    let ys = Array::from_shape_vec(
        (targets.len() as usize, 1),
        targets.iter().map(|&v| v as f64).collect(),
    ).unwrap();
    learner.train(&xs, &ys)?;

    Ok((learner.get_state(), TrainingResponse::new()))
}

fn infer(state: &LearnerState, req: &InferenceRequest) -> Result<InferenceResponse, ContractError> {
    let learner = check_owner!(state, req);

    let inputs = req.get_inputs();
    let xs = Array::from_shape_vec(
        (inputs.get_rows() as usize, inputs.get_cols() as usize),
        inputs.get_data().iter().map(|&v| v as f64).collect(),
    ).unwrap();
    let preds = learner.infer(&xs)?;

    let mut response = InferenceResponse::new();
    response.set_predictions(preds.iter().map(|&v| v as f32).collect::<Vec<f32>>());
    Ok(response)
}
