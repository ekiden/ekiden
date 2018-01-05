contract_api! {
    metadata {
        name = dummy;
        version = "0.1.0";
    }

    rpc hello_world(HelloWorldRequest) -> HelloWorldResponse;
}
