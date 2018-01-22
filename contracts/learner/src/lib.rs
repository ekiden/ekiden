#![no_std]
#![feature(prelude_import)]

#[macro_use]
extern crate sgx_tstd as std;

extern crate protobuf;
extern crate rusty_machine;
extern crate serde_cbor;

extern crate libcontract_common;
#[macro_use]
extern crate libcontract_trusted;

#[macro_use]
extern crate learner_api;

#[allow(unused)]
#[prelude_import]
use std::prelude::v1::*;

mod learner_contract;

use rusty_machine::linalg::{Matrix, Vector};

use learner_api::*;
use learner_contract::Learner;

use libcontract_common::{Address, Contract, ContractError};

create_enclave_api!();

/// Unpacks a list of `tf.Example`s into a data matrix and label vector
/// In this case, each `Example` has the format:
/// `{ (tin, a1, a2, temp_next): float_list }`
fn unpack_examples(examples: &[Example]) -> Result<(Matrix<f64>, Vector<f64>), ContractError> {
    let (x_vecs, ys_vec): (Vec<Vec<f64>>, Vec<f64>) = examples
        .iter()
        .filter_map(|example| {
            unpack_vals!(&example.get_features().feature, (tin, a1, a2, next_temp), {
                Some((vec![tin, tin * a1 - tin * a2], next_temp))
            })
        })
        .unzip();

    if x_vecs.len() == 0 {
        return Err(ContractError::new("No examples provided."));
    }

    let xs: Matrix<f64> = x_vecs.iter().map(Vec::as_slice).collect();
    let ys = Vector::new(ys_vec);

    Ok((xs, ys))
}

fn create(request: CreateRequest) -> Result<(LearnerState, CreateResponse), ContractError> {
    let learner = Learner::new(Address::from(request.get_requester().to_string()));

    Ok((learner.get_state(), CreateResponse::new()))
}

fn train(
    state: LearnerState,
    request: TrainingRequest,
) -> Result<(LearnerState, TrainingResponse), ContractError> {
    let mut learner = Learner::from_state(&state);

    if !Address::from(request.get_requester().to_string()).eq(learner.get_owner()?) {
        return Err(ContractError::new("Insufficient permissions."));
    }

    let (xs, ys) = unpack_examples(request.get_examples())?;
    let log = learner.train(&xs, &ys)?;

    let mut response = TrainingResponse::new();
    response.set_log(log);
    Ok((learner.get_state(), response))
}

fn infer(
    state: LearnerState,
    request: InferenceRequest,
) -> Result<InferenceResponse, ContractError> {
    let learner = Learner::from_state(&state);

    let (xs, _ys) = unpack_examples(request.get_examples())?;
    let preds = learner.infer(&xs)?;

    let mut response = InferenceResponse::new();
    response.set_predictions(
        preds
            .iter()
            .map(|&pred| {
                let mut pred_proto = Example::new();
                let mut pred_feature = Feature::new();
                let mut float_list = FloatList::new();
                float_list.set_value(vec![pred as f32]);
                pred_feature.set_float_list(float_list);
                pred_proto
                    .mut_features()
                    .feature
                    .insert("next_temp".to_string(), pred_feature);
                pred_proto
            })
            .collect(),
    );
    Ok(response)
}
