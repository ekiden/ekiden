extern crate futures;
extern crate futures_cpupool;
extern crate grpc;
extern crate protobuf;
extern crate tls_api;
extern crate thread_local;

#[macro_use]
extern crate clap;

extern crate libcontract_untrusted;
extern crate libcontract_common;

mod generated;
mod server;

use std::thread;

use clap::{Arg, App};
use generated::compute_web3_grpc::ComputeServer;
use server::ComputeServerImpl;

fn main() {
    let matches = App::new("Ekiden Compute Node")
                      .version("0.1.0")
                      .author("Jernej Kos <jernej@kos.mx>")
                      .about("Ekident compute node server")
                      .arg(Arg::with_name("contract")
                        .index(1)
                           .value_name("CONTRACT")
                           .help("Signed contract filename")
                           .takes_value(true)
                           .required(true)
                           .display_order(1)
                           .index(1))
                      .arg(Arg::with_name("port")
                           .long("port")
                           .short("p")
                           .takes_value(true)
                           .default_value("9001")
                           .display_order(2))
                      .get_matches();

    let port = value_t!(matches, "port", u16).unwrap_or(9001);

    // Start the gRPC server.
    let mut server = grpc::ServerBuilder::new_plain();
    server.http.set_port(port);
    server.add_service(
        ComputeServer::new_service_def(
            ComputeServerImpl::new(
                &matches.value_of("contract").unwrap()
            )
        )
    );
    server.http.set_cpu_pool_threads(1);
    let _server = server.build().expect("server");

    println!("Compute node listening at {}", port);

    loop {
        thread::park();
    }
}
