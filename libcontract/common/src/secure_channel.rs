use byteorder::{ByteOrder, LittleEndian};

use sodalite;

#[cfg(not(feature = "sgx"))]
use rand::{OsRng, Rng};

#[cfg(feature = "sgx")]
use sgx_trts;

use super::ContractError;
use super::api;

// Nonce context is used to prevent message reuse in a different context.
pub const NONCE_CONTEXT_LEN: usize = 16;
type NonceContext = [u8; NONCE_CONTEXT_LEN];
/// Nonce for use in channel initialization context (EkidenS-----Init).
pub const NONCE_CONTEXT_INIT: NonceContext = [
    69, 107, 105, 100, 101, 110, 83, 45, 45, 45, 45, 45, 73, 110, 105, 116
];
/// Nonce for use in request context (EkidenS--Request).
pub const NONCE_CONTEXT_REQUEST: NonceContext = [
    69, 107, 105, 100, 101, 110, 83, 45, 45, 82, 101, 113, 117, 101, 115, 116
];
/// Nonce for use in response context (EkidenS-Response).
pub const NONCE_CONTEXT_RESPONSE: NonceContext = [
    69, 107, 105, 100, 101, 110, 83, 45, 82, 101, 115, 112, 111, 110, 115, 101
];

/// Nonce generator.
pub trait NonceGenerator {
    /// Generate a new nonce.
    fn get_nonce(&mut self, context: &NonceContext) -> Result<sodalite::BoxNonce, ContractError>;

    /// Unpack nonce from a cryptographic box.
    fn unpack_nonce(
        &mut self,
        crypto_box: &api::CryptoBox,
        context: &NonceContext,
    ) -> Result<sodalite::BoxNonce, ContractError> {
        let mut nonce = [0u8; sodalite::BOX_NONCE_LEN];
        nonce.copy_from_slice(&crypto_box.get_nonce());

        // Ensure that the nonce context is correct.
        if nonce[..NONCE_CONTEXT_LEN] != context[..NONCE_CONTEXT_LEN] {
            return Err(ContractError::new("Invalid nonce"));
        }

        Ok(nonce)
    }
}

/// Random nonce generator.
pub struct RandomNonceGenerator {
    #[cfg(not(feature = "sgx"))]
    rng: OsRng,
}

impl RandomNonceGenerator {
    /// Create new random nonce generator.
    #[cfg(feature = "sgx")]
    pub fn new() -> Result<Self, ContractError> {
        Ok(RandomNonceGenerator {})
    }

    /// Create new random nonce generator.
    #[cfg(not(feature = "sgx"))]
    pub fn new() -> Result<Self, ContractError> {
        Ok(RandomNonceGenerator {
            rng: match OsRng::new() {
                Ok(rng) => rng,
                _ => {
                    return Err(ContractError::new(
                        "Failed to initialize random nonce generator",
                    ))
                }
            },
        })
    }
}

impl NonceGenerator for RandomNonceGenerator {
    #[cfg(feature = "sgx")]
    fn get_nonce(&mut self, context: &NonceContext) -> Result<sodalite::BoxNonce, ContractError> {
        let mut nonce: sodalite::BoxNonce = [0; sodalite::BOX_NONCE_LEN];

        match sgx_trts::rsgx_read_rand(&mut nonce) {
            Ok(_) => {}
            _ => return Err(ContractError::new("Nonce generation failed")),
        }

        nonce[..NONCE_CONTEXT_LEN].copy_from_slice(context);

        Ok(nonce)
    }

    #[cfg(not(feature = "sgx"))]
    fn get_nonce(&mut self, context: &NonceContext) -> Result<sodalite::BoxNonce, ContractError> {
        let mut nonce: sodalite::BoxNonce = [0; sodalite::BOX_NONCE_LEN];
        self.rng.fill_bytes(&mut nonce);

        nonce[..NONCE_CONTEXT_LEN].copy_from_slice(context);

        Ok(nonce)
    }
}

impl Default for RandomNonceGenerator {
    fn default() -> RandomNonceGenerator {
        RandomNonceGenerator::new().unwrap()
    }
}

/// Monotonic nonce generator.
pub struct MonotonicNonceGenerator {
    /// Next nonce to be sent.
    next_send_nonce: u64,
    /// Last nonce that was received.
    last_received_nonce: Option<u64>,
}

impl MonotonicNonceGenerator {
    /// Create new monotonic nonce generator.
    pub fn new() -> Self {
        MonotonicNonceGenerator {
            next_send_nonce: 0, // TODO: Random initialization between 0 and 2**48 - 1?
            last_received_nonce: None,
        }
    }
}

impl NonceGenerator for MonotonicNonceGenerator {
    fn get_nonce(&mut self, context: &NonceContext) -> Result<sodalite::BoxNonce, ContractError> {
        let mut nonce: Vec<u8> = context.to_vec();
        nonce.append(&mut vec![0; 8]);

        LittleEndian::write_u64(&mut nonce[NONCE_CONTEXT_LEN..], self.next_send_nonce);
        self.next_send_nonce += 1;

        assert_eq!(nonce.len(), sodalite::BOX_NONCE_LEN);

        let mut fixed_nonce: sodalite::BoxNonce = [0; sodalite::BOX_NONCE_LEN];
        fixed_nonce.copy_from_slice(&nonce);

        Ok(fixed_nonce)
    }

    fn unpack_nonce(
        &mut self,
        crypto_box: &api::CryptoBox,
        context: &NonceContext,
    ) -> Result<sodalite::BoxNonce, ContractError> {
        let mut nonce = [0u8; sodalite::BOX_NONCE_LEN];
        nonce.copy_from_slice(&crypto_box.get_nonce());

        // Ensure that the nonce context is correct.
        if nonce[..NONCE_CONTEXT_LEN] != context[..NONCE_CONTEXT_LEN] {
            return Err(ContractError::new("Invalid nonce"));
        }

        // Decode counter.
        let counter_value = LittleEndian::read_u64(&nonce[NONCE_CONTEXT_LEN..]);

        // Ensure that the nonce has increased.
        match self.last_received_nonce {
            Some(last_nonce) => {
                if counter_value <= last_nonce {
                    return Err(ContractError::new("Invalid nonce"));
                }
            }
            None => {}
        }

        self.last_received_nonce = Some(counter_value);

        Ok(nonce)
    }
}

impl Default for MonotonicNonceGenerator {
    fn default() -> MonotonicNonceGenerator {
        MonotonicNonceGenerator::new()
    }
}

/// Current state of the secure channel session.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum SessionState {
    /// Session is being initialized.
    ///
    /// From this state, the session will transition into:
    /// * `Established`.
    Init,
    /// Secure channel is established.
    Established,
}

impl SessionState {
    /// Transition secure channel to a new state.
    pub fn transition_to(&mut self, new_state: SessionState) -> Result<(), ContractError> {
        match (*self, new_state) {
            (_, SessionState::Init) => {}
            (SessionState::Init, SessionState::Established) => {}
            transition => {
                return Err(ContractError::new(&format!(
                    "Invalid secure channel state transition: {:?}",
                    transition
                )))
            }
        }

        // Update state if transition is allowed.
        *self = new_state;

        Ok(())
    }
}

impl Default for SessionState {
    fn default() -> Self {
        SessionState::Init
    }
}

/// Create cryptographic box (encrypted and authenticated).
pub fn create_box<NG: NonceGenerator>(
    payload: &[u8],
    nonce_context: &NonceContext,
    nonce_generator: &mut NG,
    public_key: &sodalite::BoxPublicKey,
    private_key: &sodalite::BoxSecretKey,
    shared_key: &mut Option<sodalite::SecretboxKey>,
) -> Result<api::CryptoBox, ContractError> {
    let mut crypto_box = api::CryptoBox::new();
    let mut key_with_payload = vec![0u8; payload.len() + 32];
    let mut encrypted = vec![0u8; payload.len() + 32];
    let nonce = nonce_generator.get_nonce(&nonce_context)?;

    // First 32 bytes is used to store the shared secret key, so we must make
    // room for it. The box_ method also requires that it is zero-initialized.
    key_with_payload[32..].copy_from_slice(payload);

    if shared_key.is_none() {
        // Compute shared key so we can speed up subsequent box operations.
        let mut key = shared_key.get_or_insert([0u8; sodalite::SECRETBOX_KEY_LEN]);
        sodalite::box_beforenm(&mut key, &public_key, &private_key);
    }

    match sodalite::box_afternm(
        &mut encrypted,
        &key_with_payload,
        &nonce,
        &shared_key.unwrap(),
    ) {
        Ok(_) => {}
        _ => return Err(ContractError::new("Box operation failed")),
    };

    crypto_box.set_nonce(nonce.to_vec());
    crypto_box.set_payload(encrypted);

    Ok(crypto_box)
}

/// Open cryptographic box.
pub fn open_box<NG: NonceGenerator>(
    crypto_box: &api::CryptoBox,
    nonce_context: &NonceContext,
    nonce_generator: &mut NG,
    public_key: &sodalite::BoxPublicKey,
    private_key: &sodalite::BoxSecretKey,
    shared_key: &mut Option<sodalite::SecretboxKey>,
) -> Result<Vec<u8>, ContractError> {
    // Reserve space for payload.
    let mut payload = vec![0u8; crypto_box.get_payload().len()];

    if shared_key.is_none() {
        // Compute shared key so we can speed up subsequent box operations.
        let mut key = shared_key.get_or_insert([0u8; sodalite::SECRETBOX_KEY_LEN]);
        sodalite::box_beforenm(&mut key, &public_key, &private_key);
    }

    match sodalite::box_open_afternm(
        &mut payload,
        &crypto_box.get_payload(),
        &nonce_generator.unpack_nonce(&crypto_box, &nonce_context)?,
        &shared_key.unwrap(),
    ) {
        Ok(_) => {
            // Trim first all-zero 32 bytes that were used to allocate space for the shared
            // secret key.
            Ok(payload[32..].to_vec())
        }
        _ => Err(ContractError::new("Failed to open box")),
    }
}
