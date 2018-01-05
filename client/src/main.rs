extern crate futures;
extern crate futures_cpupool;
extern crate protobuf;
extern crate grpc;
extern crate tls_api;

pub mod generated;

use protobuf::Message;

use generated::compute_web3::{StatusRequest, StatusResponse, CallContractRequest, CallContractResponse};
use generated::compute_web3_grpc::{Compute, ComputeClient};

use generated::contracts::token::{CreateRequest, CreateResponse};

fn main() {
    let client = ComputeClient::new_plain("localhost", 9001, Default::default()).unwrap();

    // Get compute node status.
    let request = StatusRequest::new();
    let (_, response, _) = client.status(grpc::RequestOptions::new(), request).wait().unwrap();

    let contract = response.get_contract();
    println!("Compute node is running contract '{}', version '{}'.", contract.get_name(), contract.get_version());

    if contract.get_name() != "token" {
        panic!("This client only supports the token contract.");
    }

    // TODO: Make the API below nicer to use (e.g. wrap requests automatically).

    // Create new token contract.
    let mut contract_request = CreateRequest::new();
    contract_request.set_sender("testaddr".to_string());
    contract_request.set_token_name("Ekiden Token".to_string());
    contract_request.set_token_symbol("EKI".to_string());
    contract_request.set_initial_supply(8);

    let mut request = CallContractRequest::new();
    request.set_method("create".to_string());
    request.set_payload(contract_request.write_to_bytes().unwrap());

    let (_, response, _) = client.call_contract(grpc::RequestOptions::new(), request).wait().unwrap();

    let response: CreateResponse = protobuf::parse_from_bytes(response.get_payload()).unwrap();

    println!("Response from contract: {:?}", response);
}
