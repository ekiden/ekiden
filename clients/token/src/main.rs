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
    let (state, response) = client.create(dummy_state, request).unwrap();

    println!("New state from contract: {:?}", state);
    println!("Response from contract: {:?}", response);

    let (state, response) = client.transfer(state, {
        let mut request = token::TransferRequest::new();
        request.set_sender("testaddr".to_string());
        request.set_destination("b".to_string());
        request.set_value(3);
        request
    }).unwrap();

    println!("New state from contract: {:?}", state);
    println!("Response from contract: {:?}", response);
}
