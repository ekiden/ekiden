#![feature(use_extern_macros)]

extern crate base64;
extern crate futures;
extern crate futures_cpupool;
extern crate grpc;
extern crate protobuf;
extern crate reqwest;
extern crate thread_local;
extern crate tls_api;

#[macro_use]
extern crate clap;

extern crate compute_client;
#[macro_use]
extern crate libcontract_common;
extern crate libcontract_untrusted;

mod generated;
mod ias;
mod handlers;
mod server;

use std::sync::Arc;
use std::thread;

use libcontract_common::client::ClientEndpoint;
use libcontract_untrusted::router::RpcRouter;

use clap::{App, Arg};
use generated::compute_web3_grpc::ComputeServer;
use server::ComputeServerImpl;

fn main() {
    let matches = App::new("Ekiden Compute Node")
        .version("0.1.0")
        .author("Jernej Kos <jernej@kos.mx>")
        .about("Ekident compute node server")
        .arg(
            Arg::with_name("contract")
                .index(1)
                .value_name("CONTRACT")
                .help("Signed contract filename")
                .takes_value(true)
                .required(true)
                .display_order(1)
                .index(1),
        )
        .arg(
            Arg::with_name("port")
                .long("port")
                .short("p")
                .takes_value(true)
                .default_value("9001")
                .display_order(2),
        )
        .arg(
            Arg::with_name("ias-spid")
                .long("ias-spid")
                .value_name("SPID")
                .help("IAS SPID in hex format")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("ias-pkcs12")
                .long("ias-pkcs12")
                .help("Path to IAS client certificate and private key PKCS#12 archive")
                .takes_value(true)
                .required(true),
        )
        .arg(
            Arg::with_name("key-manager-host")
                .long("key-manager-host")
                .takes_value(true)
                .default_value("localhost"),
        )
        .arg(
            Arg::with_name("key-manager-port")
                .long("key-manager-port")
                .takes_value(true)
                .default_value("9003"),
        )
        .arg(Arg::with_name("disable-key-manager").long("disable-key-manager"))
        .arg(
            Arg::with_name("grpc-threads")
                .long("grpc-threads")
                .help("Number of threads to use in the GRPC server's HTTP server. Multiple threads only allow requests to be batched up. Requests will not be processed concurrently.")
                .default_value("1")
                .takes_value(true),
        )
        .get_matches();

    let port = value_t!(matches, "port", u16).unwrap_or(9001);

    // Setup IAS.
    let ias = Arc::new(
        ias::IAS::new(ias::IASConfiguration {
            spid: value_t!(matches, "ias-spid", ias::SPID).unwrap_or_else(|e| e.exit()),
            pkcs12_archive: matches.value_of("ias-pkcs12").unwrap().to_string(),
        }).unwrap(),
    );

    // Setup enclave RPC routing.
    {
        let mut router = RpcRouter::get_mut();

        // IAS proxy endpoints.
        router.add_handler(handlers::IASProxy::new(ias.clone()));

        // Key manager endpoint.
        if !matches.is_present("disable-key-manager") {
            router.add_handler(handlers::ContractForwarder::new(
                ClientEndpoint::KeyManager,
                matches.value_of("key-manager-host").unwrap().to_string(),
                value_t!(matches, "key-manager-port", u16).unwrap_or(9003),
            ));
        }
    }

    // Start the gRPC server.
    let mut server = grpc::ServerBuilder::new_plain();
    server.http.set_port(port);
    server.add_service(ComputeServer::new_service_def(ComputeServerImpl::new(
        &matches.value_of("contract").unwrap(),
        ias.clone(),
    )));
    let num_threads = value_t!(matches, "grpc-threads", usize).unwrap();
    server.http.set_cpu_pool_threads(num_threads);
    let _server = server.build().expect("server");

    println!("Compute node listening at {}", port);

    loop {
        thread::park();
    }
}
