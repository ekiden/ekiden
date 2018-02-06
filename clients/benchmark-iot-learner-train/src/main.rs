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

extern crate learner as learner_contract; // avoid name clash with `create_client_api!` invocation
#[macro_use]
extern crate learner_api;

use clap::{App, Arg};
use learner_contract::api;

// this macro comes from learner_api
// confusingly, it creates a module called `learner` which contains the Client
create_client_api!();

const USER: &str = "Rusty Lerner";
lazy_static! {
    static ref EXAMPLES: Vec<api::Example> = {
        let mut ds_proto = std::fs::File::open(
                concat!(env!("CARGO_MANIFEST_DIR"), "/../iot_learner/iot_data.pb")
            )
            .expect("Unable to open dataset.");
        let examples_proto: api::Examples = protobuf::parse_from_reader(&mut ds_proto)
            .expect("Unable to parse dataset.");
        examples_proto.get_examples().to_vec()
    };
}

fn init<Backend>(client: &mut learner::Client<Backend>, _runs: usize, _threads: usize)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    let _create_res = client
        .create({
            let mut req = learner::CreateRequest::new();
            req.set_requester(USER.to_string());
            let inputs = vec!["tin", "tin_a1", "tin_a2"]
                .into_iter()
                .map(String::from)
                .collect();
            let targets = vec!["next_temp".to_string()];
            req.set_inputs(protobuf::RepeatedField::from_vec(inputs));
            req.set_targets(protobuf::RepeatedField::from_vec(targets));
            req
        })
        .expect("error: create");
}

fn scenario<Backend>(client: &mut learner::Client<Backend>)
where
    Backend: compute_client::backend::ContractClientBackend,
{
    let _train_res = client
        .train({
            let mut req = learner::TrainingRequest::new();
            req.set_requester(USER.to_string());
            req.set_examples(protobuf::RepeatedField::from_vec(EXAMPLES.to_vec()));
            req
        })
        .expect("error: train");
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
            contract_client!(learner, args)
        },
    );

    let results = benchmark.run(init, scenario, finalize);
    results.show();
}
