#[macro_use]
extern crate clap;
extern crate protobuf;
extern crate rulinalg;

#[macro_use]
extern crate client_utils;
#[macro_use]
extern crate compute_client;
extern crate libcontract_common;

#[macro_use]
extern crate learner_api;

use clap::{App, Arg};
use rulinalg::norm::Euclidean;
use rulinalg::vector::Vector;

use learner_api::*;

create_client_api!();

fn main() {
    let client_dir = env!("CARGO_MANIFEST_DIR").to_string();
    let contract_dir = client_dir.replace("clients", "contracts");
    let data_output = std::process::Command::new("python2")
        .arg(&(client_dir + "/src/gen_data.py"))
        .args(&[
            "--api-proto",
            &(contract_dir + "/api/src/generated/api_pb2.py"),
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
    let user = "benbitdiddle".to_string();

    let (state, _create_res) = client
        .create({
            let mut req = CreateRequest::new();
            req.set_requester(user.clone());
            req
        })
        .expect("error: create");

    let (state, _train_res) = client
        .train(state, {
            let mut req = learner::TrainingRequest::new();
            req.set_requester(user.clone());
            req.set_examples(protobuf::RepeatedField::from_vec(examples.to_vec()));
            req
        })
        .expect("error: train");

    let infer_res = client
        .infer(state, {
            let mut req = learner::InferenceRequest::new();
            req.set_requester(user.clone());
            req.set_examples(protobuf::RepeatedField::from_vec(examples.to_vec()));
            req
        })
        .expect("error: infer");

    let ground_truth: Vector<f64> = examples
        .iter()
        .filter_map(|example| unpack_val!(&example.get_features().feature, next_temp))
        .collect();

    let preds: Vector<f64> = infer_res
        .get_predictions()
        .iter()
        .filter_map(|example| unpack_val!(&example.get_features().feature, next_temp))
        .collect();

    assert!(preds.size() == ground_truth.size());

    println!(
        "Training loss: {:?}",
        (preds - ground_truth).norm(Euclidean)
    );
}
