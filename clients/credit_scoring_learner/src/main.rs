#[macro_use]
extern crate clap;
extern crate protobuf;
extern crate serde_pickle;

#[macro_use]
extern crate client_utils;
#[macro_use]
extern crate compute_client;
extern crate libcontract_common;

extern crate learner as learner_contract;

use clap::{App, Arg};
use std::process::{Command, Stdio};

use learner_contract::api::*;
use learner_contract::utils::unpack_feature_vector;

create_client_api!();

fn main() {
    let data_output = Command::new("python2")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/src/prep_data.py"))
        .output()
        .expect("Could not fetch data.");
    assert!(
        data_output.status.success(),
        "{}",
        String::from_utf8(data_output.stderr).unwrap_or("Could not generate data".to_string())
    );

    let mut examples_proto: Examples =
        protobuf::parse_from_bytes(&data_output.stdout).expect("Unable to parse Examples.");
    let (train_examples, test_examples): (Vec<Example>, Vec<Example>) = examples_proto
        .take_examples()
        .into_iter()
        .partition(|example| {
            *example
                .get_features()
                .get_feature()
                .get("is_train")
                .unwrap()
                .get_float_list()
                .get_value()
                .first()
                .unwrap() == 1.0f32
        });

    let mut client = contract_client!(learner);
    let user = "Rusty Lerner".to_string();

    let _create_res = client
        .create({
            let mut req = CreateRequest::new();
            req.set_requester(user.clone());

            let inputs = vec![
                "limit_bal",
                "bill_amt1",
                "bill_amt2",
                "bill_amt3",
                "bill_amt4",
                "bill_amt5",
                "bill_amt6",
                "pay_1",
                "pay_2",
                "pay_3",
                "pay_4",
                "pay_5",
                "pay_6",
                "pay_duly_1",
                "pay_duly_2",
                "pay_duly_3",
                "pay_duly_4",
                "pay_duly_5",
                "pay_duly_6",
                "pay_amt1",
                "pay_amt2",
                "pay_amt3",
                "pay_amt4",
                "pay_amt5",
                "pay_amt6",
                "age",
                "sex_1",
                "sex_2",
                "education_0",
                "education_1",
                "education_2",
                "education_3",
                "education_4",
                "education_5",
                "education_6",
                "marriage_0",
                "marriage_1",
                "marriage_2",
                "marriage_3",
            ].into_iter()
                .map(String::from)
                .collect();
            let targets = vec!["will_default".to_string()];
            req.set_inputs(protobuf::RepeatedField::from_vec(inputs));
            req.set_targets(protobuf::RepeatedField::from_vec(targets));
            req
        })
        .expect("error: create");

    let _train_res = client
        .train({
            let mut req = TrainingRequest::new();
            req.set_requester(user.clone());
            req.set_examples(protobuf::RepeatedField::from_vec(train_examples));
            req
        })
        .expect("error: train");

    let infer_res = client
        .infer({
            let mut req = learner::InferenceRequest::new();
            req.set_requester(user.clone());
            req.set_examples(protobuf::RepeatedField::from_vec(test_examples));
            req
        })
        .expect("error: infer");

    let preds = unpack_feature_vector(infer_res.get_predictions(), "preds").unwrap();

    let mut evaluator = Command::new("python2")
        .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/src/evaluate.py"))
        .stdin(Stdio::piped())
        .spawn()
        .expect("Could not run evaluation script.");
    serde_pickle::to_writer(
        evaluator.stdin.as_mut().unwrap(),
        preds.data(),
        false, /* use pickle 3 */
    ).expect("Could not send params.");
    evaluator.wait().expect("Evaluator script failed.");
}
