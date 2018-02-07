#[macro_use]
extern crate clap;
extern crate protobuf;
extern crate serde_pickle;

#[macro_use]
extern crate client_utils;
#[macro_use]
extern crate compute_client;
extern crate libcontract_common;

extern crate dp_credit_scoring_api as api;

use api::*;
use clap::{App, Arg};
use std::process::{Command, Stdio};

create_client_api!();

fn main() {
    let data_output = Command::new("python2")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/src/prep_data.py"))
        .args(&[
            "--api-proto",
            "/code/contracts/dp_credit_scoring/api/src/generated/api_pb2.py",
        ])
        .output()
        .expect("Could not fetch data.");
    assert!(
        data_output.status.success(),
        "{}",
        String::from_utf8(data_output.stderr).unwrap_or("Could not generate data".to_string())
    );

    let mut ds_proto: Dataset =
        protobuf::parse_from_bytes(&data_output.stdout).expect("Unable to parse Dataset.");

    let mut client = contract_client!(dp_credit_scoring);
    let user = "Rusty Lerner".to_string();

    let _create_res = client
        .create({
            let mut req = CreateRequest::new();
            req.set_requester(user.clone());
            req
        })
        .expect("error: create");

    let _train_res = client
        .train({
            let mut req = TrainingRequest::new();
            req.set_requester(user.clone());
            req.set_inputs(ds_proto.take_train_inputs());
            req.set_targets(ds_proto.take_train_targets());
            req
        })
        .expect("error: train");

    let mut infer_res = client
        .infer({
            let mut req = InferenceRequest::new();
            req.set_requester(user.clone());
            req.set_inputs(ds_proto.take_test_inputs());
            req
        })
        .expect("error: infer");

    let preds = (infer_res.take_predictions(), ds_proto.take_test_targets());

    let mut evaluator = Command::new("python2")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/src/evaluate.py"))
        .stdin(Stdio::piped())
        .spawn()
        .expect("Could not run evaluation script.");
    serde_pickle::to_writer(
        evaluator.stdin.as_mut().unwrap(),
        &preds,
        false, /* use pickle 3 */
    ).expect("Could not send predictions.");
    evaluator.wait().expect("Evaluator script failed.");
}
