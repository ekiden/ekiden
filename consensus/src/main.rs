#[macro_use]
extern crate clap;
extern crate consensus;

use clap::{App, Arg};

fn main() {
    let matches = App::new("Ekiden Compute Node")
        .version("0.1.0")
        .about("Ekiden consensus node")
        .arg(
            Arg::with_name("tendermint-host")
                .long("tendermint-host")
                .takes_value(true)
                .default_value("localhost"),
        )
        .arg(
            Arg::with_name("tendermint-port")
                .long("tendermint-port")
                .takes_value(true)
                .default_value("46657"),
        )
        .arg(
            Arg::with_name("tendermint-abci-port")
                .long("tendermint-abci-port")
                .takes_value(true)
                .default_value("46658"),
        )
        .arg(
            Arg::with_name("grpc-port")
                .long("grpc-port")
                .takes_value(true)
                .default_value("9002"),
        )
        .arg(
            Arg::with_name("no-tendermint")
                .long("no-tendermint")
                .short("x")
        )
        .get_matches();

    let config = consensus::Config {
        tendermint_host: matches.value_of("tendermint-host").unwrap().to_string(),
        tendermint_port: value_t!(matches, "tendermint-port", u16).unwrap_or_else(|e| e.exit()),
        tendermint_abci_port: value_t!(matches, "tendermint-abci-port", u16)
            .unwrap_or_else(|e| e.exit()),
        grpc_port: value_t!(matches, "grpc-port", u16).unwrap_or_else(|e| e.exit()),
        no_tendermint: { matches.occurrences_of("no-tendermint") > 0 },
    };

    println!(
        "Ekiden Consensus Node starting on port {} ... ",
        config.grpc_port
    );
    if let Err(e) = consensus::run(&config) {
        println!("Application error: {}", e);
        std::process::exit(1);
    }
}
