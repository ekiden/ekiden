#![no_std]
#![feature(prelude_import)]

#[macro_use]
extern crate sgx_tstd as std;

extern crate protobuf;
extern crate rusty_machine;

extern crate libcontract_common;
#[macro_use]
extern crate libcontract_trusted;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

#[macro_use]
extern crate learner;

pub use libcontract_common::{Address, Contract, ContractError};

use rusty_machine::learning::logistic_reg::LogisticRegressor;
use rusty_machine::learning::optim::grad_desc::GradientDesc;
use rusty_machine::prelude::*;

use learner::Learner;
use learner::api::*;
use learner::utils::{pack_proto, unpack_feature_matrix, unpack_feature_vector};

create_enclave_api!();

type Model = LogisticRegressor<GradientDesc>;

fn create(req: &CreateRequest) -> Result<(LearnerState, CreateResponse), ContractError> {
    let learner = Learner::new(
        Address::from(req.get_requester().to_string()),
        LogisticRegressor::default(),
        req.get_inputs().to_vec(),
        req.get_targets().to_vec(),
    )?;
    Ok((learner.get_state(), CreateResponse::new()))
}

fn train(
    state: &LearnerState,
    req: &TrainingRequest,
) -> Result<(LearnerState, TrainingResponse), ContractError> {
    let mut learner = check_owner!(Model, state, req);

    let examples = req.get_examples();
    let xs = unpack_feature_matrix(examples, learner.get_inputs()?)?;
    let ys = unpack_target_vec!(examples, learner.get_targets()?)?;
    learner.train(&xs, &ys)?;

    Ok((learner.get_state(), TrainingResponse::new()))
}

fn infer(state: &LearnerState, req: &InferenceRequest) -> Result<InferenceResponse, ContractError> {
    let learner = check_owner!(Model, state, req);

    let xs = unpack_feature_matrix(req.get_examples(), learner.get_inputs()?)?;
    let preds = learner.infer(&xs)?;

    let mut response = InferenceResponse::new();
    response.set_predictions(pack_proto(vec![
        ("preds".to_string(), Matrix::from(preds)),
    ])?);
    Ok(response)
}
