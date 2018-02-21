#[macro_use]
extern crate clap;
extern crate futures;
extern crate protobuf;
extern crate rulinalg;
extern crate tokio_core;

#[macro_use]
extern crate client_utils;
#[macro_use]
extern crate compute_client;
extern crate libcontract_common;

extern crate learner as learner_contract;
use clap::{App, Arg};
use futures::Future;
use rulinalg::norm::Euclidean;
use rulinalg::vector::Vector;

// this macro comes from learner_api
// confusingly, it creates a module called `learner` which contains the Client
create_client_api!();
use learner_contract::api::*;

use learner_contract::utils::unpack_feature_vector;

fn main() {
    let data_output = std::process::Command::new("python2")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/src/gen_data.py"))
        .args(&[
            // "--api-proto",
            // "/code/contracts/learner/api/src/generated/api_pb2.py",
            "--data-proto",
            "/code/clients/iot_learner/iot_data.pb",
        ])
        .output()
        .expect("Could not fetch data.");
    assert!(
        data_output.status.success(),
        "{}",
        String::from_utf8(data_output.stderr).unwrap_or("Could not generate data".to_string())
    );

    let examples_proto: Examples =
        protobuf::parse_from_bytes(&data_output.stdout).expect("Unable to parse Examples.");
    let examples = examples_proto.get_examples();

    let mut client = contract_client!(learner);
    let user = "Rusty Lerner".to_string();

    let _create_res = client
        .create({
            let mut req = learner::CreateRequest::new();
            req.set_requester(user.clone());
            let inputs = vec!["tin", "tin_a1", "tin_a2"]
                .into_iter()
                .map(String::from)
                .collect();
            let targets = vec!["next_temp".to_string()];
            req.set_inputs(protobuf::RepeatedField::from_vec(inputs));
            req.set_targets(protobuf::RepeatedField::from_vec(targets));
            req
        })
        .wait()
        .expect("error: create");

    let _train_res = client
        .train({
            let mut req = learner::TrainingRequest::new();
            req.set_requester(user.clone());
            req.set_examples(protobuf::RepeatedField::from_vec(examples.to_vec()));
            req
        })
        .wait()
        .expect("error: train");

    let infer_res = client
        .infer({
            let mut req = learner::InferenceRequest::new();
            req.set_requester(user.clone());
            req.set_examples(protobuf::RepeatedField::from_vec(examples.to_vec()));
            req
        })
        .wait()
        .expect("error: infer");

    let ground_truth: Vector<f64> = unpack_feature_vector(examples, "next_temp").unwrap();
    let preds: Vector<f64> = unpack_feature_vector(infer_res.get_predictions(), "preds").unwrap();

    assert!(preds.size() == ground_truth.size());

    println!(
        "Training loss: {:?}",
        (preds - ground_truth).norm(Euclidean)
    );
}
