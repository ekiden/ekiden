extern crate consensus;

use std::thread;

//use super::generated::consensus;
//use super::generated::consensus_grpc;
//use super::generated::consensus_grpc::Consensus;
//use grpc;

#[test]
fn processes_requests() {
    let config = consensus::Config {
        tendermint_host: String::from("localhost"),
        tendermint_port: 46657,
        tendermint_abci_port: 46658,
        grpc_port: 9002,
    };

    let server_handle = thread::spawn(move || {
        consensus::run(&config).unwrap();
    });
    
    assert_eq!(4, 5);

    //let consensus_client =
        //    consensus_grpc::ConsensusClient::new_plain("localhost", 9002, Default::default())
        //        .unwrap();

        //// Set state to `helloworld`
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
    server_handle.join();
}
