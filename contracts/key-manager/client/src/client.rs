use std::collections::HashMap;
use std::collections::hash_map::Entry;
#[cfg(not(target_env = "sgx"))]
use std::sync::{Mutex, MutexGuard};
#[cfg(target_env = "sgx")]
use std::sync::SgxMutex as Mutex;
#[cfg(target_env = "sgx")]
use std::sync::SgxMutexGuard as MutexGuard;

use ekiden_common::error::{Error, Result};
use ekiden_enclave_common::quote::{MrEnclave, MRENCLAVE_LEN};
use ekiden_rpc_client::{create_client_rpc, FutureExtra};
use ekiden_rpc_common::client::ClientEndpoint;
use ekiden_rpc_trusted::client::OcallContractClientBackend;

use key_manager_api::with_api;

// Create API client for the key manager.
with_api! {
    create_client_rpc!(key_manager, key_manager_api, api);
}

/// Key manager client interface.
pub struct KeyManager {
    /// Internal API client.
    client: Option<key_manager::Client<OcallContractClientBackend>>,
    /// Local key cache.
    cache: HashMap<String, Vec<u8>>,
}

lazy_static! {
    // Global key store object.
    static ref KEY_MANAGER: Mutex<KeyManager> = Mutex::new(KeyManager::new());
}

impl KeyManager {
    /// Key manager contract MRENCLAVE.
    const MR_ENCLAVE: MrEnclave = MrEnclave(*include_bytes!("generated/key_manager_mrenclave.bin"));

    /// Construct new key manager interface.
    fn new() -> Self {
        KeyManager {
            client: None,
            cache: HashMap::new(),
        }
    }

    /// Establish a connection with the key manager contract.
    ///
    /// This will establish a mutually authenticated secure channel with the key manager
    /// contract, so this operation may fail due to the key manager being unavailable or
    /// issues with establishing a mutually authenticated secure channel.
    fn connect(&mut self) -> Result<()> {
        if KeyManager::is_self() {
            return Err(Error::new(
                "Tried to call key manager from inside the key manager itself",
            ));
        }

        if self.client.is_some() {
            return Ok(());
        }

        let backend = match OcallContractClientBackend::new(ClientEndpoint::KeyManager) {
            Ok(backend) => backend,
            _ => return Err(Error::new("Failed to create key manager client backend")),
        };

        let client = key_manager::Client::new(backend, KeyManager::MR_ENCLAVE);
        self.client.get_or_insert(client);

        Ok(())
    }

    /// Get global key manager client instance.
    ///
    /// Calling this method will take a lock on the global instance, which will
    /// be released once the value goes out of scope.
    pub fn get<'a>() -> Result<MutexGuard<'a, KeyManager>> {
        let mut manager = KEY_MANAGER.lock().unwrap();

        // Ensure manager is connected.
        manager.connect()?;

        Ok(manager)
    }

    /// Checks if the client is running inside the key manager itself.
    ///
    /// This should be used to prevent the key manager contract from trying to also
    /// contact the key manager. This determination is based on the MRENCLAVE being
    /// all zeroes in the key manager contract itself.
    pub fn is_self() -> bool {
        KeyManager::MR_ENCLAVE == MrEnclave([0; MRENCLAVE_LEN])
    }

    /// Clear local key cache.
    ///
    /// This will make the client re-fetch the keys from the key manager.
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get or create named key.
    ///
    /// If the key does not yet exist, the key manager will generate one. If
    /// the key has already been cached locally, it will be retrieved from
    /// cache.
    pub fn get_or_create_key(&mut self, name: &str, size: usize) -> Result<Vec<u8>> {
        // Check cache first.
        match self.cache.entry(name.to_string()) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(entry) => {
                // No entry in cache, fetch from key manager.
                let mut request = key_manager::GetOrCreateKeyRequest::new();
                request.set_name(name.to_string());
                request.set_size(size as u32);

                let mut response = match self.client
                    .as_mut()
                    .unwrap()
                    .get_or_create_key(request)
                    .wait()
                {
                    Ok(response) => response,
                    Err(error) => {
                        return Err(Error::new(&format!(
                            "Failed to call key manager: {}",
                            error.message
                        )))
                    }
                };

                Ok(entry.insert(response.take_key()).clone())
            }
        }
    }
}
