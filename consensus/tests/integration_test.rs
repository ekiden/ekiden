extern crate consensus as lib;
extern crate grpc;

use std::thread;

use lib::generated::consensus;
use lib::generated::consensus_grpc;
use lib::generated::consensus_grpc::Consensus;

#[test]
fn processes_requests() {
    let config = lib::Config {
        tendermint_host: String::from("localhost"),
        tendermint_port: 46657,
        tendermint_abci_port: 46658,
        grpc_port: 9002,
    };
    let client_port = config.grpc_port;

    let server_handle = thread::spawn(move || {
        lib::run(&config).unwrap();
    });

    let client =
        consensus_grpc::ConsensusClient::new_plain("localhost", client_port, Default::default())
            .unwrap();

    // Get latest state - should be empty
    let req = consensus::GetRequest::new();
    let (_, resp, _) = client.get(grpc::RequestOptions::new(), req).wait().unwrap();
    assert_eq!(
        resp.get_checkpoint().get_payload(),
        String::from("helloworld").as_bytes()
    );

    // Set state to `helloworld`
    //let mut consensus_set_request = consensus::SetRequest::new();
    //consensus_set_request.set_payload(String::from("helloworld").into_bytes());
    //consensus_client
    //    .set(grpc::RequestOptions::new(), consensus_set_request)
    //    .wait()
    //    .unwrap();

    //let consensus_get_request = consensus::GetRequest::new();
    //let (_, consensus_get_response, _) = consensus_client
    //    .get(grpc::RequestOptions::new(), consensus_get_request)
    //    .wait()
    //    .unwrap();
    //assert_eq!(
    //    consensus_get_response.get_payload(),
    //    String::from("helloworld").as_bytes()
    //);

    //// Set state to `successor`
    //let mut consensus_set_request = consensus::SetRequest::new();
    //consensus_set_request.set_payload(String::from("successor").into_bytes());
    //consensus_client
    //    .set(grpc::RequestOptions::new(), consensus_set_request)
    //    .wait()
    //    .unwrap();

    //let consensus_get_request = consensus::GetRequest::new();
    //let (_, consensus_get_response, _) = consensus_client
    //    .get(grpc::RequestOptions::new(), consensus_get_request)
    //    .wait()
    //    .unwrap();
    //assert_eq!(
    //    consensus_get_response.get_payload(),
    //    String::from("successor").as_bytes()
    //);

    //// Set state to a sequence of all byte values
    //let mut scale: Vec<u8> = vec![0; 256];
    //for i in 0..256 {
    //    scale[i] = i as u8;
    //}

    //let mut consensus_set_request = consensus::SetRequest::new();
    //consensus_set_request.set_payload(scale.clone());
    //consensus_client
    //    .set(grpc::RequestOptions::new(), consensus_set_request)
    //    .wait()
    //    .unwrap();

    //let consensus_get_request = consensus::GetRequest::new();
    //let (_, consensus_get_response, _) = consensus_client
    //    .get(grpc::RequestOptions::new(), consensus_get_request)
    //    .wait()
    //    .unwrap();
    //assert_eq!(consensus_get_response.get_payload(), &scale[..]);

    // See https://github.com/sunblaze-ucb/ekiden/issues/223
    // We can't gracefully shut down the server yet.
    assert_eq!(4, 5);
    server_handle.join();
}
