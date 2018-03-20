//! Secure channel handling.
#[cfg(target_env = "sgx")]
use sgx_tseal::SgxSealedData;
#[cfg(target_env = "sgx")]
use sgx_types::*;

use protobuf;
use protobuf::Message;
use sodalite;

use std::collections::HashMap;
#[cfg(not(target_env = "sgx"))]
use std::sync::{Mutex, MutexGuard};
#[cfg(target_env = "sgx")]
use std::sync::SgxMutex as Mutex;
#[cfg(target_env = "sgx")]
use std::sync::SgxMutexGuard as MutexGuard;

use ekiden_common::error::{Error, Result};
use ekiden_common::random;
use ekiden_enclave_common::quote::{AttestationReport, MrEnclave, QUOTE_CONTEXT_SC};
use ekiden_rpc_common::api;
use ekiden_rpc_common::secure_channel::{self, MonotonicNonceGenerator, RandomNonceGenerator,
                                        SessionState};

use super::quote::create_attestation_report_for_public_key;
use super::request::Request;

// Secret seed used for generating private and public keys.
const SECRET_SEED_LEN: usize = 32;
type SecretSeed = [u8; SECRET_SEED_LEN];

/// Single secure channel session between client and contract.
#[derive(Default)]
pub struct ClientSession {
    /// Client short-term public key.
    client_public_key: sodalite::BoxPublicKey,
    /// Contract short-term public key.
    contract_public_key: sodalite::BoxPublicKey,
    /// Contract short-term private key.
    contract_private_key: sodalite::BoxSecretKey,
    /// Cached shared key.
    shared_key: Option<sodalite::SecretboxKey>,
    /// Short-term nonce generator.
    nonce_generator: MonotonicNonceGenerator,
    /// Session state.
    state: SessionState,
    /// Client MRENCLAVE (if attested).
    client_mr_enclave: Option<MrEnclave>,
}

/// Secure channel context.
pub struct SecureChannelContext {
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
    /// Current attestation report.
    attestation_report: AttestationReport,
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
            nonce_generator: RandomNonceGenerator::new(),
            attestation_report: AttestationReport::default(),
        }
    }

    /// Global secure channel context instance.
    ///
    /// Calling this method will take a lock on the global instance which
    /// will be released once the value goes out of scope.
    pub fn get<'a>() -> MutexGuard<'a, Self> {
        SECURE_CHANNEL_CTX.lock().unwrap()
    }

    /// Channel readiness status.
    pub fn is_ready(&self) -> bool {
        self.ready
    }

    /// Checks channel readiness status.
    fn ensure_ready(&self) -> Result<()> {
        if !self.ready {
            return Err(Error::new("Secure channel not ready"));
        }

        Ok(())
    }

    /// Configure a keypair for the secure channel.
    fn set_keypair(&mut self, seed: &SecretSeed) -> Result<()> {
        // Ignore requests if channel already initialized.
        if self.ready {
            return Err(Error::new("Secure channel already initialized"));
        }

        sodalite::box_keypair_seed(&mut self.public_key, &mut self.private_key, &seed);
        self.seed = seed.clone();

        // Keypair has been changed, we need to refresh the attestation report.
        #[cfg(target_env = "sgx")]
        self.refresh_attestation_report()?;

        self.ready = true;

        Ok(())
    }

    /// Generate and configure a new random keypair for the secure channel.
    pub fn generate_keypair(&mut self) -> Result<()> {
        let mut seed: SecretSeed = [0; SECRET_SEED_LEN];
        match random::get_random_bytes(&mut seed) {
            Ok(_) => {}
            Err(_) => return Err(Error::new("Keypair generation failed")),
        }

        self.set_keypair(&seed)?;

        Ok(())
    }

    /// Unseal and configure a keypair for the secure channel.
    #[cfg(target_env = "sgx")]
    pub fn unseal_keypair(&mut self, sealed_keys: &[u8]) -> Result<()> {
        let sealed_data = unsafe {
            SgxSealedData::<SecretSeed>::from_raw_sealed_data_t(
                sealed_keys.as_ptr() as *mut sgx_sealed_data_t,
                sealed_keys.len() as u32,
            )
        };

        match sealed_data {
            Some(data) => {
                let unsealed_data = match data.unseal_data() {
                    Ok(data) => data,
                    Err(_) => return Err(Error::new("Failed to unseal keypair")),
                };

                self.set_keypair(unsealed_data.get_decrypt_txt())?;

                Ok(())
            }
            None => Err(Error::new("Failed to unseal keypair")),
        }
    }

    #[cfg(not(target_env = "sgx"))]
    pub fn unseal_keypair(&mut self, _sealed_keys: &[u8]) -> Result<()> {
        Err(Error::new("Only supported in SGX builds"))
    }

    /// Generate a fresh attestation report for IAS.
    pub fn refresh_attestation_report(&mut self) -> Result<()> {
        self.attestation_report = create_attestation_report_for_public_key(
            &QUOTE_CONTEXT_SC,
            &[0; 16],
            &self.get_public_key(),
        )?;

        Ok(())
    }

    /// Return current attestation report.
    pub fn get_attestation_report(&self) -> &AttestationReport {
        &self.attestation_report
    }

    /// Get contract long-term public key.
    pub fn get_public_key(&self) -> &sodalite::BoxPublicKey {
        &self.public_key
    }

    /// Return sealed keypair.
    #[cfg(target_env = "sgx")]
    pub fn get_sealed_keypair(&self) -> Result<Vec<u8>> {
        let void: [u8; 0] = [0_u8; 0];
        let sealed_data = match SgxSealedData::<SecretSeed>::seal_data_ex(
            0x01, // KEYPOLICY_MRENCLAVE
            sgx_attributes_t {
                flags: 0xfffffffffffffff3,
                xfrm: 0,
            },
            0xF0000000,
            &void,
            &self.seed,
        ) {
            Ok(data) => data,
            Err(_) => return Err(Error::new("Failed to seal keypair")),
        };

        let raw_data_len = SgxSealedData::<SecretSeed>::calc_raw_sealed_data_size(
            sealed_data.get_add_mac_txt_len(),
            sealed_data.get_encrypt_txt_len(),
        );
        let mut raw_data: Vec<u8> = vec![];
        raw_data.resize(raw_data_len as usize, 0);

        unsafe {
            sealed_data
                .to_raw_sealed_data_t(raw_data.as_ptr() as *mut sgx_sealed_data_t, raw_data_len)
        };

        Ok(raw_data)
    }

    #[cfg(not(target_env = "sgx"))]
    pub fn get_sealed_keypair(&self) -> Result<Vec<u8>> {
        Err(Error::new("Only supported in SGX builds"))
    }

    /// Convert client short-term public key into session hash map key.
    fn get_session_key(public_key: &[u8]) -> Result<sodalite::BoxPublicKey> {
        if public_key.len() != sodalite::BOX_PUBLIC_KEY_LEN {
            return Err(Error::new("Bad short-term client key"));
        }

        let mut key: sodalite::BoxPublicKey = [0; sodalite::BOX_PUBLIC_KEY_LEN];
        key.copy_from_slice(&public_key);

        Ok(key)
    }

    /// Create a new client session.
    ///
    /// Returns a cryptographic box, encrypted to the client short-term key and
    /// authenticated by the contract long-term key.
    pub fn create_session(
        &mut self,
        public_key: &[u8],
        client_attestation: Option<AttestationReport>,
    ) -> Result<api::CryptoBox> {
        self.ensure_ready()?;

        let key = SecureChannelContext::get_session_key(&public_key)?;

        if self.sessions.contains_key(&key) {
            return Err(Error::new("Session already exists"));
        }

        let mut session = ClientSession::new(key.clone())?;
        let mut response_box = api::ChannelInitResponseBox::new();
        response_box.set_short_term_public_key(session.get_contract_public_key().to_vec());

        // Verify client attestation when required.
        match client_attestation {
            Some(report) => session.verify_client_attestation(&report)?,
            None => {}
        }

        session.transition_to(SessionState::Established)?;

        let mut shared_key: Option<sodalite::SecretboxKey> = None;
        let crypto_box = secure_channel::create_box(
            response_box.write_to_bytes()?.as_slice(),
            &secure_channel::NONCE_CONTEXT_INIT,
            &mut self.nonce_generator,
            session.get_client_public_key(),
            &self.private_key,
            &mut shared_key,
        )?;

        // TODO: What about session table overflows?

        self.sessions.insert(key, session);

        Ok(crypto_box)
    }

    /// Lookup existing client session.
    pub fn get_session(&mut self, public_key: &[u8]) -> Result<&mut ClientSession> {
        self.ensure_ready()?;

        let key = SecureChannelContext::get_session_key(&public_key)?;

        match self.sessions.get_mut(&key) {
            Some(session) => Ok(session),
            None => Err(Error::new("Client session not found")),
        }
    }

    /// Close an existing session.
    pub fn close_session(&mut self, public_key: &[u8]) -> Result<()> {
        let key = SecureChannelContext::get_session_key(&public_key)?;

        self.sessions.remove(&key);

        Ok(())
    }
}

impl ClientSession {
    /// Create a new client session.
    pub fn new(public_key: sodalite::BoxPublicKey) -> Result<Self> {
        let mut session = ClientSession::default();
        session.transition_to(SessionState::Init)?;
        session.client_public_key = public_key;

        // Generate new keypair.
        let mut seed: SecretSeed = [0; SECRET_SEED_LEN];
        match random::get_random_bytes(&mut seed) {
            Ok(_) => {}
            Err(_) => return Err(Error::new("Keypair generation failed")),
        }

        sodalite::box_keypair_seed(
            &mut session.contract_public_key,
            &mut session.contract_private_key,
            &seed,
        );

        // Cache shared channel key.
        {
            let mut key = session
                .shared_key
                .get_or_insert([0u8; sodalite::SECRETBOX_KEY_LEN]);
            sodalite::box_beforenm(
                &mut key,
                &session.client_public_key,
                &session.contract_private_key,
            );
        }

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
    pub fn open_request_box(&mut self, request: &api::CryptoBox) -> Result<Request<Vec<u8>>> {
        let plain_request = secure_channel::open_box(
            &request,
            &secure_channel::NONCE_CONTEXT_REQUEST,
            &mut self.nonce_generator,
            &self.client_public_key,
            &self.contract_private_key,
            &mut self.shared_key,
        )?;

        let mut plain_request: api::PlainClientRequest =
            protobuf::parse_from_bytes(&plain_request)?;

        // Check if this request is allowed based on current channel state.
        match self.state {
            SessionState::Established => {}
            _ => {
                return Err(Error::new("Invalid method call in this state"));
            }
        }

        Ok(Request::new(
            plain_request.take_payload(),
            plain_request.take_method(),
            Some(self.client_public_key.to_vec()),
            self.client_mr_enclave.clone(),
        ))
    }

    /// Create cryptographic box with RPC response.
    pub fn create_response_box(
        &mut self,
        response: &api::PlainClientResponse,
    ) -> Result<api::CryptoBox> {
        Ok(secure_channel::create_box(
            &response.write_to_bytes()?,
            &secure_channel::NONCE_CONTEXT_RESPONSE,
            &mut self.nonce_generator,
            &self.client_public_key,
            &self.contract_private_key,
            &mut self.shared_key,
        )?)
    }

    /// Verify client attestation.
    pub fn verify_client_attestation(&mut self, report: &AttestationReport) -> Result<()> {
        let quote = report.get_quote()?;

        if quote.get_quote_context() != QUOTE_CONTEXT_SC {
            return Err(Error::new("Client attestation failed: invalid context"));
        }

        if quote.get_public_key() != self.client_public_key {
            return Err(Error::new("Client attestation failed: invalid public key"));
        }

        // Extract MRENCLAVE.
        self.client_mr_enclave = Some(quote.get_mr_enclave().clone());

        Ok(())
    }

    /// Transition secure channel to a new state.
    pub fn transition_to(&mut self, new_state: SessionState) -> Result<()> {
        Ok(self.state.transition_to(new_state)?)
    }
}

lazy_static! {
    // Global secure channel context.
    static ref SECURE_CHANNEL_CTX: Mutex<SecureChannelContext> =
        Mutex::new(SecureChannelContext::new());
}

/// Initialize contract.
pub fn contract_init(_request: &api::ContractInitRequest) -> Result<api::ContractInitResponse> {
    let mut channel = SECURE_CHANNEL_CTX.lock().unwrap();

    // Generate a new keypair.
    channel.generate_keypair()?;

    let mut response = api::ContractInitResponse::new();
    response.set_public_key(channel.get_public_key().to_vec());
    response.set_sealed_keys(channel.get_sealed_keypair()?);
    response.set_mr_enclave(
        channel
            .get_attestation_report()
            .get_quote()?
            .get_mr_enclave()
            .0
            .to_vec(),
    );

    Ok(response)
}

/// Restore contract from sealed state.
pub fn contract_restore(
    request: &api::ContractRestoreRequest,
) -> Result<api::ContractRestoreResponse> {
    let mut channel = SECURE_CHANNEL_CTX.lock().unwrap();

    // Unseal existing keypair.
    channel.unseal_keypair(request.get_sealed_keys())?;

    let mut response = api::ContractRestoreResponse::new();
    response.set_public_key(channel.get_public_key().to_vec());

    Ok(response)
}

/// Initialize secure channel.
///
/// If the `client_attestation_required` is set to `true`, then the initialization
/// request must contain a valid client attestation report.
pub fn channel_init(request: &api::ChannelInitRequest) -> Result<api::ChannelInitResponse> {
    let mut channel = SECURE_CHANNEL_CTX.lock().unwrap();

    channel.ensure_ready()?;

    let client_attestation = if request.has_client_attestation_report() {
        // Verify attestation report.
        let report = request.get_client_attestation_report();
        let report = AttestationReport::new(
            report.get_body().to_vec(),
            report.get_signature().to_vec(),
            report.get_certificates().to_vec(),
        );
        report.verify()?;

        Some(report)
    } else {
        None
    };

    // Create new session.
    let response_box =
        channel.create_session(request.get_short_term_public_key(), client_attestation)?;

    // Serialize attestation report.
    let report = channel.get_attestation_report();
    let mut serialized_report = api::AttestationReport::new();
    serialized_report.set_body(report.body.clone());
    serialized_report.set_signature(report.signature.clone());
    serialized_report.set_certificates(report.certificates.clone());

    let mut response = api::ChannelInitResponse::new();
    response.set_contract_attestation_report(serialized_report);
    response.set_response_box(response_box);

    Ok(response)
}

/// Close secure channel.
pub fn channel_close(public_key: &[u8]) -> Result<()> {
    let mut channel = SECURE_CHANNEL_CTX.lock().unwrap();

    channel.close_session(&public_key)?;

    Ok(())
}

/// Open cryptographic box with RPC request.
pub fn open_request_box(request: &api::CryptoBox) -> Result<Request<Vec<u8>>> {
    let mut channel = SECURE_CHANNEL_CTX.lock().unwrap();

    Ok(channel
        .get_session(&request.get_public_key())?
        .open_request_box(&request)?)
}

/// Create cryptographic box with RPC response.
pub fn create_response_box(
    public_key: &[u8],
    response: &api::PlainClientResponse,
) -> Result<api::CryptoBox> {
    let mut channel = SECURE_CHANNEL_CTX.lock().unwrap();

    Ok(channel
        .get_session(&public_key)?
        .create_response_box(&response)?)
}
