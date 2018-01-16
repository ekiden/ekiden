contract_api! {
    metadata {
        name = dummy;
        version = "0.1.0";
        state_type = protobuf::well_known_types::Empty;
    }

    rpc hello_world(HelloWorldRequest) -> HelloWorldResponse;
}
