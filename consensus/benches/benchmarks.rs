#![feature(test)]

extern crate consensus as lib;
extern crate grpc;
extern crate test;

use std::{thread, time};
use test::Bencher;

use lib::generated::consensus;
use lib::generated::consensus_grpc;
use lib::generated::consensus_grpc::Consensus;

#[bench]
fn benchmark_get(b: &mut Bencher) {
    let config = lib::Config {
        tendermint_host: String::from("localhost"),
        tendermint_port: 46657,
        tendermint_abci_port: 46658,
        grpc_port: 9002,
    };
    let client_port = config.grpc_port;

    let _server_handle = thread::spawn(move || {
        lib::run(&config).unwrap();
    });

    // Give time for Tendermint to connect
    thread::sleep(time::Duration::from_millis(3000));

    let client =
        consensus_grpc::ConsensusClient::new_plain("localhost", client_port, Default::default())
            .unwrap();

    // Set state to `helloworld`
    let mut req = consensus::ReplaceRequest::new();
    req.set_payload(String::from("helloworld").into_bytes());
    client
        .replace(grpc::RequestOptions::new(), req)
        .wait()
        .unwrap();

    b.iter(move || {
        let req = consensus::GetRequest::new();
        let (_, resp, _) = client.get(grpc::RequestOptions::new(), req).wait().unwrap();
        assert_eq!(
            resp.get_checkpoint().get_payload(),
            String::from("helloworld").as_bytes()
        );
    });

    // See https://github.com/sunblaze-ucb/ekiden/issues/223
    // We can't gracefully shut down the server yet.
    // panic!("Test passed, just need to panic to get out");
    //server_handle.join();
}
