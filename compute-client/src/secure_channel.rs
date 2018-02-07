use sodalite;

use protobuf;
use protobuf::Message;

use libcontract_common::{api, random};
use libcontract_common::secure_channel::{create_box, open_box, MonotonicNonceGenerator,
                                         NonceGenerator, RandomNonceGenerator, SessionState,
                                         NONCE_CONTEXT_INIT, NONCE_CONTEXT_REQUEST,
                                         NONCE_CONTEXT_RESPONSE};

use super::errors::Error;

// Secret seed used for generating private and public keys.
const SECRET_SEED_LEN: usize = 32;
type SecretSeed = [u8; SECRET_SEED_LEN];

/// Secure channel context.
///
/// Contains state and methods needed for secure communication with the remote
/// contract.
#[derive(Default)]
pub struct SecureChannelContext {
    /// Client short-term private key.
    client_private_key: sodalite::BoxSecretKey,
    /// Client short-term public key.
    client_public_key: sodalite::BoxPublicKey,
    /// Contract contract long-term public key.
    contract_long_term_public_key: sodalite::BoxPublicKey,
    /// Contract contract short-term public key.
    contract_short_term_public_key: sodalite::BoxPublicKey,
    /// Cached shared key.
    shared_key: Option<sodalite::SecretboxKey>,
    /// Session state.
    state: SessionState,
    /// Long-term nonce generator.
    long_term_nonce_generator: RandomNonceGenerator,
    /// Short-term nonce generator.
    short_term_nonce_generator: MonotonicNonceGenerator,
}

impl SecureChannelContext {
    /// Reset secure channel context.
    ///
    /// Calling this function will generate new short-term keys for the client
    /// and clear any contract public keys.
    pub fn reset(&mut self) -> Result<(), Error> {
        // Generate new short-term key pair for the client.
        let mut seed: SecretSeed = [0u8; SECRET_SEED_LEN];
        random::get_random_bytes(&mut seed)?;

        sodalite::box_keypair_seed(
            &mut self.client_public_key,
            &mut self.client_private_key,
            &seed,
        );

        // Clear contract keys.
        self.contract_long_term_public_key = [0; sodalite::BOX_PUBLIC_KEY_LEN];
        self.contract_short_term_public_key = [0; sodalite::BOX_PUBLIC_KEY_LEN];

        // Clear session keys.
        self.shared_key = None;

        // Reset session nonce.
        self.short_term_nonce_generator.reset();

        self.state.transition_to(SessionState::Init)?;

        Ok(())
    }

    /// Setup secure channel.
    pub fn setup(
        &mut self,
        contract_long_term_public_key: &[u8],
        contract_response_box: &api::CryptoBox,
    ) -> Result<(), Error> {
        self.contract_long_term_public_key
            .copy_from_slice(&contract_long_term_public_key);

        // Open boxed short term server public key.
        let mut shared_key: Option<sodalite::SecretboxKey> = None;
        let response_box = open_box(
            &contract_response_box,
            &NONCE_CONTEXT_INIT,
            &mut self.long_term_nonce_generator,
            &self.contract_long_term_public_key,
            &self.client_private_key,
            &mut shared_key,
        )?;

        let response_box: api::ChannelInitResponseBox = protobuf::parse_from_bytes(&response_box)?;

        self.contract_short_term_public_key
            .copy_from_slice(&response_box.get_short_term_public_key());

        self.state.transition_to(SessionState::Established)?;

        // Cache shared channel key.
        let mut key = self.shared_key
            .get_or_insert([0u8; sodalite::SECRETBOX_KEY_LEN]);
        sodalite::box_beforenm(
            &mut key,
            &self.contract_short_term_public_key,
            &self.client_private_key,
        );

        Ok(())
    }

    /// Close secure channel.
    ///
    /// After the secure channel is closed, it must be reset to be used again.
    pub fn close(&mut self) {
        self.state.transition_to(SessionState::Closed).unwrap();
    }

    /// Check if secure channel is closed.
    pub fn is_closed(&self) -> bool {
        self.state == SessionState::Closed
    }

    /// Check if messages must be encrypted based on current channel state.
    ///
    /// Messages can only be unencrypted when the channel is in initialization state
    /// and must be encrypted in all other states.
    pub fn must_encrypt(&self) -> bool {
        self.state == SessionState::Established
    }

    /// Get client short-term public key.
    pub fn get_client_public_key(&self) -> &sodalite::BoxPublicKey {
        &self.client_public_key
    }

    /// Create cryptographic box with RPC request.
    pub fn create_request_box(
        &mut self,
        request: &api::PlainClientRequest,
    ) -> Result<api::CryptoBox, Error> {
        let mut crypto_box = create_box(
            &request.write_to_bytes()?,
            &NONCE_CONTEXT_REQUEST,
            &mut self.short_term_nonce_generator,
            &self.contract_short_term_public_key,
            &self.client_private_key,
            &mut self.shared_key,
        )?;

        // Set public key so the contract knows which client this is.
        crypto_box.set_public_key(self.client_public_key.to_vec());

        Ok(crypto_box)
    }

    /// Open cryptographic box with RPC response.
    pub fn open_response_box(
        &mut self,
        response: &api::CryptoBox,
    ) -> Result<api::PlainClientResponse, Error> {
        let plain_response = open_box(
            &response,
            &NONCE_CONTEXT_RESPONSE,
            &mut self.short_term_nonce_generator,
            &self.contract_short_term_public_key,
            &self.client_private_key,
            &mut self.shared_key,
        )?;

        Ok(protobuf::parse_from_bytes(&plain_response)?)
    }
}
