#[macro_use]
extern crate compute_client;

#[macro_use]
extern crate token_api;

#[macro_use]
extern crate clap;

use clap::{Arg, App};

create_client_api!();

fn main() {
    let matches = App::new("Ekiden Token Contract Client")
                      .version("0.1.0")
                      .author("Jernej Kos <jernej@kos.mx>")
                      .about("Client for the Ekiden Token Contract")
                      .arg(Arg::with_name("host")
                           .long("host")
                           .short("h")
                           .takes_value(true)
                           .default_value("localhost")
                           .display_order(1))
                      .arg(Arg::with_name("port")
                           .long("port")
                           .short("p")
                           .takes_value(true)
                           .default_value("9001")
                           .display_order(2))
                      .arg(Arg::with_name("ias-spid")
                           .long("ias-spid")
                           .value_name("SPID")
                           .help("IAS SPID in hex format")
                           .takes_value(true)
                           .requires("ias-pkcs12"))
                      .arg(Arg::with_name("ias-pkcs12")
                           .long("ias-pkcs12")
                           .help("Path to IAS client certificate and private key PKCS#12 archive")
                           .takes_value(true)
                           .requires("ias-spid"))
                      .get_matches();

    let mut client = token::Client::new(
        matches.value_of("host").unwrap(),
        value_t!(matches, "port", u16).unwrap_or(9001),
        if matches.is_present("ias-spid") {
            Some(compute_client::IASConfiguration {
                spid: value_t!(matches, "ias-spid", compute_client::SPID).unwrap_or_else(|e| e.exit()),
                pkcs12_archive: matches.value_of("ias-pkcs12").unwrap().to_string()
            })
        } else {
            None
        }
    ).unwrap();

    // Create new token contract.
    let mut request = token::CreateRequest::new();
    request.set_sender("testaddr".to_string());
    request.set_token_name("Ekiden Token".to_string());
    request.set_token_symbol("EKI".to_string());
    request.set_initial_supply(8);

    let response = client.create(request).unwrap();

    println!("Response from contract: {:?}", response);
}
