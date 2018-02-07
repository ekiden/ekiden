#[macro_use]
extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate protobuf;

#[macro_use]
extern crate client_utils;
#[macro_use]
extern crate compute_client;
extern crate libcontract_common;

extern crate dp_credit_scoring_api as api;

use api::*;
use clap::{App, Arg};
use std::process::Command;

create_client_api!();

const USER: &str = "Rusty Lerner";
lazy_static! {
    static ref DATASET: Dataset = {
        let data_output = Command::new("python2")
            .arg(concat!(env!("CARGO_MANIFEST_DIR"), "/../dp_credit_scoring/src/prep_data.py"))
            .args(&[
                "--api-proto",
                "/code/contracts/dp_credit_scoring/api/src/generated/api_pb2.py",
            ])
            .args(&["--max-samples", "32"])
            .output()
            .expect("Could not fetch data.");
        assert!(
            data_output.status.success(),
            "{}",
            String::from_utf8(data_output.stderr).unwrap_or("Could not generate data".to_string())
        );

        protobuf::parse_from_bytes(&data_output.stdout).expect("Unable to parse Dataset.")
    };
}

fn init<Backend>(client: &mut dp_credit_scoring::Client<Backend>, _runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    let _create_res = client
        .create({
            let mut req = CreateRequest::new();
            req.set_requester(USER.to_string());
            req
        })
        .expect("error: create");
}

fn scenario<Backend>(client: &mut dp_credit_scoring::Client<Backend>)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    let _train_res = client
        .train({
            let mut req = TrainingRequest::new();
            req.set_requester(USER.to_string());
            req.set_inputs(DATASET.get_train_inputs().clone());
            req.set_targets(DATASET.get_train_targets().to_vec());
            req
        })
        .expect("error: train");
}

fn finalize<Backend>(
    _client: &mut dp_credit_scoring::Client<Backend>,
    _runs: usize,
    _threads: usize,
) where
    Backend: compute_client::backend::ContractClientBackend,
{
    // No actions.
}

fn main() {
    let args = std::sync::Arc::new(
        default_app!()
            .arg(
                Arg::with_name("benchmark-threads")
                    .long("benchmark-threads")
                    .help("Number of benchmark threads")
                    .takes_value(true)
                    .default_value("4"),
            )
            .arg(
                Arg::with_name("benchmark-runs")
                    .long("benchmark-runs")
                    .help("Number of scenario runs")
                    .takes_value(true)
                    .default_value("1000"),
            )
            .get_matches(),
    );

    let benchmark = client_utils::benchmark::Benchmark::new(
        value_t!(args, "benchmark-runs", usize).unwrap_or_else(|e| e.exit()),
        value_t!(args, "benchmark-threads", usize).unwrap_or_else(|e| e.exit()),
        move || {
            let args = args.clone();
            contract_client!(dp_credit_scoring, args)
        },
    );

    let results = benchmark.run(init, scenario, finalize);
    results.show();
}
