contract_api! {
    metadata {
        name = token;
        version = "0.1.0";
    }

    rpc create(CreateRequest) -> CreateResponse;

    rpc transfer(TransferRequest) -> TransferResponse;
}
