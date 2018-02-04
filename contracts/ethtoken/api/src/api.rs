contract_api! {
    metadata {
        name = ethtoken;
        version = "0.1.0";
        state_type = EthState;
        client_attestation_required = false;
    }

    rpc init_genesis_state(InitStateRequest) -> (state, InitStateResponse);

    rpc create_token(state, CreateTokenRequest) -> (state, CreateTokenResponse);

    rpc transfer_tokens(state, TransferTokenRequest) -> (state, TransferTokenResponse);

    rpc get_balance(state, GetBalanceRequest) -> GetBalanceResponse;
}
