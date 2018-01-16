contract_api! {
    metadata {
        name = token;
        version = "0.1.0";
        state_type = TokenState;
    }

    rpc create(CreateRequest) -> (state, CreateResponse);

    rpc transfer(state, TransferRequest) -> (state, TransferResponse);
}
