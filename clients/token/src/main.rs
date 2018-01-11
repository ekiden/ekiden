#[macro_use]
extern crate compute_client;

#[macro_use]
extern crate token_api;

// TODO: remove this when client doesn't have to deal with state
extern crate protobuf;
use protobuf::Message;

create_client_api!();

fn main() {
    let client = token::Client::new("localhost", 9001).unwrap();

    // Create new token contract.
    let mut request = token::CreateRequest::new();
    request.set_sender("testaddr".to_string());
    request.set_token_name("Ekiden Token".to_string());
    request.set_token_symbol("EKI".to_string());
    request.set_initial_supply(8);

    let dummy_state = token_api::TokenState::default_instance().write_to_bytes().unwrap();
    let (new_state, response) = client.create(dummy_state, request).unwrap();

    println!("New state from contract: {:?}", new_state);
    println!("Response from contract: {:?}", response);
}
