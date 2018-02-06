
use clap::{App, Arg};

fn main() {
    let matches = App::new("Ekiden Compute Node")
        .version("0.1.0")
        .about("Ekident consensus node")
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
        .get_matches();

    println!("Ekiden Consensus starting... ");

    let port = value_t!(matches, "grpc-port", u16).unwrap_or_else(|e| e.exit());
    println!("Consensus node listening at {}", port);
}

