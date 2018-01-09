use sgx_types::*;
use sgx_trts;
use sgx_tseal::SgxSealedData;

use sodalite;

use std::sync::SgxMutex;

use libcontract_common::api;
use libcontract_common::ContractError;

const SECRET_SEED_LEN: usize = 32;
type SecretSeed = [u8; SECRET_SEED_LEN];

/// Secure channel context.
struct SecureChannelContext {
    seed: SecretSeed,
    public_key: sodalite::BoxPublicKey,
    private_key: sodalite::BoxSecretKey,
    ready: bool,
}

impl SecureChannelContext {
    /// Create new secure channel context.
    pub fn new() -> Self {
        SecureChannelContext {
            seed: [0; SECRET_SEED_LEN],
            public_key: [0; sodalite::BOX_PUBLIC_KEY_LEN],
            private_key: [0; sodalite::BOX_SECRET_KEY_LEN],
            ready: false,
        }
    }

    /// Configure a keypair for the secure channel.
    fn set_keypair(&mut self, seed: &SecretSeed) -> Result<(), ContractError> {
        // Ignore requests if channel already initialized.
        if self.ready {
            return Err(ContractError::new("Secure channel already initialized"));
        }

        sodalite::box_keypair_seed(
            &mut self.public_key,
            &mut self.private_key,
            &seed
        );
        self.seed = seed.clone();
        self.ready = true;

        Ok(())
    }

    /// Generate and configure a new random keypair for the secure channel.
    pub fn generate_keypair(&mut self) -> Result<(), ContractError> {
        let mut seed: SecretSeed = [0; SECRET_SEED_LEN];
        match sgx_trts::rsgx_read_rand(&mut seed) {
            Ok(_) => {},
            Err(_) => return Err(ContractError::new("Keypair generation failed"))
        }

        self.set_keypair(&seed)?;

        Ok(())
    }

    /// Unseal and configure a keypair for the secure channel.
    pub fn unseal_keypair(&mut self, sealed_keys: &[u8]) -> Result<(), ContractError> {
        let sealed_data = unsafe {
            SgxSealedData::<SecretSeed>::from_raw_sealed_data_t(
                sealed_keys.as_ptr() as * mut sgx_sealed_data_t,
                sealed_keys.len() as u32
            )
        };

        match sealed_data {
            Some(data) => {
                let unsealed_data = match data.unseal_data() {
                    Ok(data) =>  data,
                    Err(_) =>  return Err(ContractError::new("Failed to unseal keypair"))
                };

                self.set_keypair(unsealed_data.get_decrypt_txt())?;

                Ok(())
            },
            None => Err(ContractError::new("Failed to unseal keypair"))
        }
    }

    /// Return public key.
    pub fn get_public_key(&self) -> Vec<u8> {
        self.public_key.to_vec()
    }

    /// Return sealed keypair.
    pub fn get_sealed_keypair(&self) -> Result<Vec<u8>, ContractError> {
        let void: [u8; 0] = [0_u8; 0];
        let sealed_data = match SgxSealedData::<SecretSeed>::seal_data(&void, &self.seed) {
            Ok(data) => data,
            Err(_) => return Err(ContractError::new("Failed to seal keypair"))
        };

        let raw_data_len = SgxSealedData::<SecretSeed>::calc_raw_sealed_data_size(
            sealed_data.get_add_mac_txt_len(),
            sealed_data.get_encrypt_txt_len()
        );
        let mut raw_data: Vec<u8> = vec![];
        raw_data.resize(raw_data_len as usize, 0);

        unsafe {
            sealed_data.to_raw_sealed_data_t(
                raw_data.as_ptr() as * mut sgx_sealed_data_t,
                raw_data_len
            )
        };

        Ok(raw_data)
    }
}

lazy_static! {
    // Global secure channel context.
    static ref SECURE_CHANNEL_CTX: SgxMutex<SecureChannelContext> = SgxMutex::new(SecureChannelContext::new());
}

/// Initialize contract.
pub fn contract_init(request: api::ContractInitRequest) -> Result<api::ContractInitResponse, ContractError> {

    let mut channel = SECURE_CHANNEL_CTX.lock().unwrap();

    if request.get_sealed_keys().is_empty() {
        // Generate a new keypair.
        channel.generate_keypair()?;
    } else {
        // Unseal existing keypair.
        channel.unseal_keypair(request.get_sealed_keys())?;
    }

    let mut response = api::ContractInitResponse::new();
    response.set_public_key(channel.get_public_key());
    response.set_sealed_keys(channel.get_sealed_keypair()?);

    Ok(response)
}
