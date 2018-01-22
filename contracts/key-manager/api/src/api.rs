contract_api! {
    metadata {
        name = key_manager;
        version = "0.1.0";
        state_type = protobuf::well_known_types::Empty;
        client_attestation_required = true;
    }

    rpc get_or_create_key(GetOrCreateKeyRequest) -> GetOrCreateKeyResponse;
}
