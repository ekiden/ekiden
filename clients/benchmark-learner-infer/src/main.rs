#[macro_use]
extern crate clap;
extern crate protobuf;

#[macro_use]
extern crate client_utils;
#[macro_use]
extern crate compute_client;
extern crate libcontract_common;

extern crate learner_api as api;

use api::*;
use clap::{App, Arg};

create_client_api!();

const USER: &str = "Rusty Lerner";
static mut DS_PROTO: Option<Dataset> = None;

fn init<Backend>(client: &mut learner::Client<Backend>, _runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    println!("SENDING CREATE");

    let _create_res = client
        .create({
            let mut req = CreateRequest::new();
            req.set_requester(USER.to_owned());
            req
        })
        .expect("error: create");

    let ds_ref = unsafe {
        // Safe because not mutations will happen during the scenario runs.
        DS_PROTO.as_ref().unwrap()
    };

    println!("SENDING TRAIN");

    let _train_res = client
        .train({
            let mut req = TrainingRequest::new();
            req.set_requester(USER.to_owned());
            req.set_inputs(ds_ref.get_train_inputs().clone());
            req.set_targets(ds_ref.get_train_targets().to_vec());
            req
        })
        .expect("error: train");
}

fn scenario<Backend>(client: &mut learner::Client<Backend>)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    let ds_ref = unsafe {
        // Safe because not mutations will happen during the scenario runs.
        DS_PROTO.as_ref().unwrap()
    };

    println!("HERE");

    let mut _infer_res = client
        .infer({
            let mut req = learner::InferenceRequest::new();
            req.set_requester(USER.to_owned());
            req.set_inputs(ds_ref.get_test_inputs().clone());
            req
        })
        .expect("error: infer");
}

fn finalize<Backend>(_client: &mut learner::Client<Backend>, _runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    // No actions.
}

fn main() {
    let args = std::sync::Arc::new(
        default_app!()
            .arg(Arg::with_name("benchmark-threads")
                 .long("benchmark-threads")
                 .help("Number of benchmark threads")
                 .takes_value(true)
                 .default_value("4"))
            .arg(Arg::with_name("benchmark-runs")
                 .long("benchmark-runs")
                 .help("Number of scenario runs")
                 .takes_value(true)
                 .default_value("1000"))
            .arg(Arg::with_name("data")
                 .long("data")
                 .help("A file with the output of prep_data.py from the learner contract")
                 .takes_value(true)
                 .required(true))
            .get_matches()
    );

    let data_filename = value_t!(args, "data", String).unwrap();
    unsafe {
        // Safe because we have exclusive access at this time.
        DS_PROTO = Some(protobuf::parse_from_reader(&mut std::fs::File::open(data_filename).expect("Unable to open dataset.")).expect("Unable to parse dataset."));
    }

    let benchmark = client_utils::benchmark::Benchmark::new(
        value_t!(args, "benchmark-runs", usize).unwrap_or_else(|e| e.exit()),
        value_t!(args, "benchmark-threads", usize).unwrap_or_else(|e| e.exit()),
        move || {
            let args = args.clone();
            contract_client!(learner, args)
        }
    );

    let results = benchmark.run(init, scenario, finalize);
    results.show();
}
