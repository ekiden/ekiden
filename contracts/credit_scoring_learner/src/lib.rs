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

#[macro_use(check_owner, unpack_target_vec)]
extern crate learner;

#[macro_use]
extern crate credit_scoring_learner_api;

pub use libcontract_common::{Address, Contract, ContractError};

use rusty_machine::learning::logistic_reg::LogisticRegressor;
use rusty_machine::learning::optim::grad_desc::GradientDesc;
use rusty_machine::prelude::*;

use learner::Learner;
use learner::api::{CreateRequest, CreateResponse, InferenceRequest, InferenceResponse,
                   LearnerState, TrainingRequest, TrainingResponse};
use learner::utils::{pack_proto, unpack_feature_matrix, unpack_feature_vector};

use credit_scoring_learner_api::*;

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

fn get_parameters(
    state: &LearnerState,
    req: &ParametersRequest,
) -> Result<ParametersResponse, ContractError> {
    let learner = check_owner!(Model, state, req);

    let params = learner
        .get_model()?
        .parameters()
        .ok_or(ContractError::new("Model hasn't been trained."))?;

    let mut response = ParametersResponse::new();
    response.set_parameters(params.iter().map(|&v| v as f32).collect());
    Ok(response)
}
