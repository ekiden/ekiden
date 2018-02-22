rpc_api! {
    metadata {
        name = key_manager;
        version = "0.1.0";
        client_attestation_required = true;
    }

    rpc get_or_create_key(GetOrCreateKeyRequest) -> GetOrCreateKeyResponse;
}
