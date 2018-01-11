contract_api! {
    metadata {
        name = dummy;
        version = "0.1.0";
        state_type = libcontract_common::api::Void;
    }

    rpc hello_world(HelloWorldRequest) -> HelloWorldResponse;
}
