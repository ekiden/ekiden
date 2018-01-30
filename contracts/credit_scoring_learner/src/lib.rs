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
use rusty_machine::linalg;

use learner::Learner;
use learner::api::*;

create_enclave_api!();

type Model = LogisticRegressor<GradientDesc>;

fn create(req: &CreateRequest) -> Result<(LearnerState, CreateResponse), ContractError> {
    let learner = Learner::new(
        Address::from(req.get_requester().to_string()),
        LogisticRegressor::default(),
    )?;
    Ok((learner.get_state(), CreateResponse::new()))
}

fn train(
    state: &LearnerState,
    req: &TrainingRequest,
) -> Result<(LearnerState, TrainingResponse), ContractError> {
    let mut learner = check_owner!(Model, state, req);

    let inputs = req.get_inputs();
    let xs: linalg::Matrix<f64> = linalg::Matrix::new(
        inputs.get_rows() as usize,
        inputs.get_cols() as usize,
        inputs
            .get_data()
            .to_vec()
            .iter()
            .map(|&v| v as f64)
            .collect::<Vec<f64>>(),
    );
    let ys = linalg::Vector::new(
        req.get_targets()
            .iter()
            .map(|&v| v as f64)
            .collect::<Vec<f64>>(),
    );
    learner.train(&xs, &ys)?;

    Ok((learner.get_state(), TrainingResponse::new()))
}

fn infer(state: &LearnerState, req: &InferenceRequest) -> Result<InferenceResponse, ContractError> {
    let learner = check_owner!(Model, state, req);

    let inputs = req.get_inputs();
    let xs = linalg::Matrix::new(
        inputs.get_rows() as usize,
        inputs.get_cols() as usize,
        inputs
            .get_data()
            .iter()
            .map(|&v| v as f64)
            .collect::<Vec<f64>>(),
    );
    let preds = learner.infer(&xs)?;

    let mut response = InferenceResponse::new();
    response.set_predictions(preds.data().iter().map(|&v| v as f32).collect::<Vec<f32>>());
    Ok(response)
}
