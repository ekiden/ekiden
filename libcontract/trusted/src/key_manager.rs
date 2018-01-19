use libcontract_common::ContractError;
use libcontract_common::client::ClientEndpoint;

use compute_client;
use compute_client::create_client;

use key_manager_api::create_client_api as create_key_manager_client_api;

use super::client::OcallContractClientBackend;

// Create API client for the key manager.
create_key_manager_client_api!();

// TODO: Key manager contract MRENCLAVE.
// TODO: Import this from file generated during build.

/// Key manager interface.
struct KeyManager {
    /// Internal API client.
    client: key_manager::Client<OcallContractClientBackend>,
}

impl KeyManager {
    pub fn new() -> Result<Self, ContractError> {
        let backend = match OcallContractClientBackend::new(ClientEndpoint::KeyManager) {
            Ok(backend) => backend,
            _ => {
                return Err(ContractError::new(
                    "Failed to create key manager client backend",
                ))
            }
        };

        let client = match key_manager::Client::new(
            backend,
            // TODO: Get MRENCLAVE from file generated during build.
            compute_client::MrEnclave([0; 32]),
        ) {
            Ok(client) => client,
            _ => return Err(ContractError::new("Failed to create key manager client")),
        };

        Ok(KeyManager { client: client })
    }

    // TODO: Key manager operations.
}
