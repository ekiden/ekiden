use sgx_types::*;
use sgx_trts;
use sgx_tse;
use sgx_tseal::SgxSealedData;

use protobuf;
use protobuf::Message;
use sodalite;

use std::sync::SgxMutex;
use std::collections::HashMap;

use libcontract_common::{api, secure_channel, ContractError};
use libcontract_common::secure_channel::{RandomNonceGenerator, MonotonicNonceGenerator};

use super::untrusted;

// Secret seed used for generating private and public keys.
const SECRET_SEED_LEN: usize = 32;
type SecretSeed = [u8; SECRET_SEED_LEN];

#[derive(Default)]
struct ClientSession {
    /// Client short-term public key.
    client_public_key: sodalite::BoxPublicKey,
    /// Contract short-term public key.
    contract_public_key: sodalite::BoxPublicKey,
    /// Contract short-term private key.
    contract_private_key: sodalite::BoxSecretKey,
    /// Short-term nonce generator.
    nonce_generator: MonotonicNonceGenerator,
}

/// Secure channel context.
struct SecureChannelContext {
    /// Secret seed used to generate server public and private keys.
    seed: SecretSeed,
    /// Public server key.
    public_key: sodalite::BoxPublicKey,
    /// Private server key.
    private_key: sodalite::BoxSecretKey,
    /// Readiness of the channel.
    ready: bool,
    /// Contract short-term keypairs, keyed with client short-term keys.
    sessions: HashMap<sodalite::BoxPublicKey, ClientSession>,
    /// Long-term nonce generator.
    nonce_generator: RandomNonceGenerator,
}

impl SecureChannelContext {
    /// Create new secure channel context.
    pub fn new() -> Self {
        SecureChannelContext {
            seed: [0; SECRET_SEED_LEN],
            public_key: [0; sodalite::BOX_PUBLIC_KEY_LEN],
            private_key: [0; sodalite::BOX_SECRET_KEY_LEN],
            ready: false,
            sessions: HashMap::new(),
            nonce_generator: RandomNonceGenerator::new().unwrap(),
        }
    }

    /// Checks channel readiness status.
    fn ensure_ready(&self) -> Result<(), ContractError> {
        if !self.ready {
            return Err(ContractError::new("Secure channel not ready"));
        }

        Ok(())
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

    /// Get contract long-term public key.
    pub fn get_public_key(&self) -> &sodalite::BoxPublicKey {
        &self.public_key
    }

    /// Return sealed keypair.
    pub fn get_sealed_keypair(&self) -> Result<Vec<u8>, ContractError> {
        let void: [u8; 0] = [0_u8; 0];
        let sealed_data = match SgxSealedData::<SecretSeed>::seal_data_ex(
            0x01, // KEYPOLICY_MRENCLAVE
            sgx_attributes_t {
                flags: 0xfffffffffffffff3,
                xfrm: 0
            },
            0xF0000000,
            &void,
            &self.seed
        ) {
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

    /// Convert client short-term public key into session hash map key.
    fn get_session_key(public_key: &[u8]) -> Result<sodalite::BoxPublicKey, ContractError> {
        if public_key.len() != sodalite::BOX_PUBLIC_KEY_LEN {
            return Err(ContractError::new("Bad short-term client key"));
        }

        let mut key: sodalite::BoxPublicKey = [0; sodalite::BOX_PUBLIC_KEY_LEN];
        key.copy_from_slice(&public_key);

        Ok(key)
    }

    /// Create a new client session.
    ///
    /// Returns a cryptographic box, encrypted to the client short-term key and
    /// authenticated by the contract long-term key.
    pub fn create_session(&mut self, public_key: &[u8]) -> Result<api::CryptoBox, ContractError> {
        self.ensure_ready()?;

        let key = SecureChannelContext::get_session_key(&public_key)?;

        if self.sessions.contains_key(&key) {
            return Err(ContractError::new("Session already exists"));
        }

        let session = ClientSession::new(key.clone())?;
        let crypto_box = secure_channel::create_box(
            session.get_contract_public_key(),
            &secure_channel::NONCE_CONTEXT_INIT,
            &mut self.nonce_generator,
            session.get_client_public_key(),
            &self.private_key
        )?;

        // TODO: What about session table overflows?

        self.sessions.insert(key, session);

        Ok(crypto_box)
    }

    /// Lookup existing client session.
    pub fn get_session(&mut self, public_key: &[u8]) -> Result<&mut ClientSession, ContractError> {
        self.ensure_ready()?;

        let key = SecureChannelContext::get_session_key(&public_key)?;

        match self.sessions.get_mut(&key) {
            Some(session) => Ok(session),
            None => Err(ContractError::new("Client session not found"))
        }
    }

    /// Close an existing session.
    pub fn close_session(&mut self, public_key: &[u8]) -> Result<(), ContractError> {
        let key = SecureChannelContext::get_session_key(&public_key)?;

        self.sessions.remove(&key);

        Ok(())
    }
}

impl ClientSession {
    /// Create a new client session.
    pub fn new(public_key: sodalite::BoxPublicKey) -> Result<Self, ContractError> {
        let mut session = ClientSession::default();
        session.client_public_key = public_key;

        // Generate new keypair.
        let mut seed: SecretSeed = [0; SECRET_SEED_LEN];
        match sgx_trts::rsgx_read_rand(&mut seed) {
            Ok(_) => {},
            Err(_) => return Err(ContractError::new("Keypair generation failed"))
        }

        sodalite::box_keypair_seed(
            &mut session.contract_public_key,
            &mut session.contract_private_key,
            &seed
        );

        Ok(session)
    }

    /// Get client short-term public key.
    pub fn get_client_public_key(&self) -> &sodalite::BoxPublicKey {
        &self.client_public_key
    }

    /// Get contract short-term public key.
    pub fn get_contract_public_key(&self) -> &sodalite::BoxPublicKey {
        &self.contract_public_key
    }

    /// Open cryptographic box with RPC request.
    pub fn open_request_box(&mut self, request: &api::CryptoBox) -> Result<api::PlainRequest, ContractError> {
        let plain_request = secure_channel::open_box(
            &request,
            &secure_channel::NONCE_CONTEXT_REQUEST,
            &mut self.nonce_generator,
            &self.client_public_key,
            &self.contract_private_key
        )?;

        Ok(protobuf::parse_from_bytes(&plain_request)?)
    }

    /// Create cryptographic box with RPC response.
    pub fn create_response_box(&mut self, response: &api::PlainResponse) -> Result<api::CryptoBox, ContractError> {
        Ok(secure_channel::create_box(
            &response.write_to_bytes()?,
            &secure_channel::NONCE_CONTEXT_RESPONSE,
            &mut self.nonce_generator,
            &self.client_public_key,
            &self.contract_private_key
        )?)
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
    response.set_public_key(channel.get_public_key().to_vec());
    response.set_sealed_keys(channel.get_sealed_keypair()?);

    Ok(response)
}

macro_rules! sgx_call {
    ($error: expr, $result: ident, $block: block) => {
        let status = unsafe { $block };

        match status {
            sgx_status_t::SGX_SUCCESS => {
                match $result {
                    sgx_status_t::SGX_SUCCESS => {},
                    _ => return Err(ContractError::new($error))
                };
            },
            _ => return Err(ContractError::new($error))
        };
    }
}

/// Initialize secure channel.
pub fn channel_init(request: api::ChannelInitRequest) -> Result<api::ChannelInitResponse, ContractError> {

    // Validate request.
    if request.get_nonce().len() != 16 {
        return Err(ContractError::new("Invalid nonce"));
    } else if request.get_spid().len() != 16 {
        return Err(ContractError::new("Invalid SPID"));
    }

    let mut channel = SECURE_CHANNEL_CTX.lock().unwrap();

    channel.ensure_ready()?;

    // Initialize target suitable for use by the quoting enclave.
    let mut result = sgx_status_t::SGX_ERROR_UNEXPECTED;
    let mut target_info = sgx_target_info_t::default();
    let mut epid_group = sgx_epid_group_id_t::default();

    sgx_call!("Failed to initialize quote", result, {
        untrusted::untrusted_init_quote(
            &mut result,
            &mut target_info as * mut sgx_target_info_t,
            &mut epid_group as * mut sgx_epid_group_id_t
        )
    });

    // Generate report for the quoting enclave (include channel public key in report data).
    let mut report_data = sgx_report_data_t::default();
    let pkey_len = sodalite::BOX_PUBLIC_KEY_LEN;
    report_data.d[..pkey_len].copy_from_slice(channel.get_public_key());
    report_data.d[pkey_len..pkey_len + 16].copy_from_slice(&request.get_nonce()[..16]);

    let report = match sgx_tse::rsgx_create_report(&target_info, &report_data) {
        Ok(report) => report,
        _ => return Err(ContractError::new("Failed to create report"))
    };

    // Request the quoting enclave to generate a quote from our report.
    let mut qe_report = sgx_report_t::default();
    let mut qe_nonce = sgx_quote_nonce_t { rand: [0; 16] };
    let mut spid = sgx_spid_t { id: [0; 16] };

    // Maximum quote size is 16K.
    let mut quote: Vec<u8> = Vec::with_capacity(16 * 1024);
    let mut quote_size = 0;

    spid.id.copy_from_slice(&request.get_spid()[..16]);

    match sgx_trts::rsgx_read_rand(&mut qe_nonce.rand) {
        Ok(_) => {},
        _ => return Err(ContractError::new("Failed to generate random nonce"))
    };

    sgx_call!("Failed to get quote", result, {
        untrusted::untrusted_get_quote(
            &mut result,
            &report as * const sgx_report_t,
            sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
            &spid as * const sgx_spid_t,
            &qe_nonce as * const sgx_quote_nonce_t,
            &mut qe_report as * mut sgx_report_t,
            quote.as_mut_ptr() as * mut u8,
            quote.capacity() as u32,
            &mut quote_size
        )
    });

    match sgx_tse::rsgx_verify_report(&qe_report) {
        Ok(_) => {},
        _ => return Err(ContractError::new("Failed to get quote"))
    };

    unsafe {
        quote.set_len(quote_size as usize);
    }

    // TODO: Verify QE signature. Note that this may not be the QE enclave at all as
    // untrusted_init_quote can provide an arbitrary enclave target. Is there a way
    // to get the QE identity in a secure way?
    // lower 32Bytes in report.data = SHA256(qe_nonce||quote).

    // Create new session.
    let crypto_box = channel.create_session(request.get_short_term_public_key())?;

    let mut response = api::ChannelInitResponse::new();
    response.set_quote(quote);
    response.set_short_term_public_key(crypto_box);

    Ok(response)
}

/// Close secure channel.
pub fn channel_close(public_key: &[u8]) -> Result<(), ContractError> {
    let mut channel = SECURE_CHANNEL_CTX.lock().unwrap();

    channel.close_session(&public_key)?;

    Ok(())
}

/// Open cryptographic box with RPC request.
pub fn open_request_box(request: &api::CryptoBox) -> Result<api::PlainRequest, ContractError> {
    let mut channel = SECURE_CHANNEL_CTX.lock().unwrap();

    Ok(channel.get_session(&request.get_public_key())?
              .open_request_box(&request)?)
}

/// Create cryptographic box with RPC response.
pub fn create_response_box(public_key: &[u8],
                           response: &api::PlainResponse) -> Result<api::CryptoBox, ContractError> {

    let mut channel = SECURE_CHANNEL_CTX.lock().unwrap();

    Ok(channel.get_session(&public_key)?
              .create_response_box(&response)?)
}
