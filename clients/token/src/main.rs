#[macro_use]
extern crate compute_client;

#[macro_use]
extern crate token_api;

create_client_api!();

fn main() {
    let client = token::Client::new("localhost", 9001).unwrap();

    // Create new token contract.
    let mut request = token::CreateRequest::new();
    request.set_sender("testaddr".to_string());
    request.set_token_name("Ekiden Token".to_string());
    request.set_token_symbol("EKI".to_string());
    request.set_initial_supply(8);

    let dummy_state = Vec::new();
    let (new_state, response) = client.create(dummy_state, request).unwrap();

    println!("New state from contract: {:?}", new_state);
    println!("Response from contract: {:?}", response);
}
