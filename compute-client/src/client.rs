use sodalite;

use protobuf;
use protobuf::{Message, MessageStatic};

use libcontract_common::{api, random};
use libcontract_common::quote::MrEnclave;
use libcontract_common::secure_channel::{create_box, open_box, MonotonicNonceGenerator,
                                         RandomNonceGenerator, SessionState, NONCE_CONTEXT_INIT,
                                         NONCE_CONTEXT_REQUEST, NONCE_CONTEXT_RESPONSE};

use super::backend::ContractClientBackend;
use super::errors::Error;

// Secret seed used for generating private and public keys.
const SECRET_SEED_LEN: usize = 32;
type SecretSeed = [u8; SECRET_SEED_LEN];

/// Secure channel context.
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
    /// Cached shared request key.
    shared_request_key: Option<sodalite::SecretboxKey>,
    /// Cached shared response key.
    shared_response_key: Option<sodalite::SecretboxKey>,
    /// Session state.
    state: SessionState,
    /// Long-term nonce generator.
    long_term_nonce_generator: RandomNonceGenerator,
    /// Short-term nonce generator.
    short_term_nonce_generator: MonotonicNonceGenerator,
    /// Client attestation nonce.
    client_attestation_nonce: Vec<u8>,
    /// Client attestation SPID.
    client_attestation_spid: Vec<u8>,
}

/// Contract client.
pub struct ContractClient<Backend: ContractClientBackend> {
    /// Backend handling network communication.
    backend: Backend,
    /// Contract MRENCLAVE.
    mr_enclave: MrEnclave,
    /// Secure channel context.
    secure_channel: SecureChannelContext,
}

impl<Backend: ContractClientBackend> ContractClient<Backend> {
    /// Constructs a new contract client.
    pub fn new(backend: Backend, mr_enclave: MrEnclave) -> Result<Self, Error> {
        let mut client = ContractClient {
            backend: backend,
            mr_enclave: mr_enclave,
            secure_channel: SecureChannelContext::default(),
        };

        // Initialize a secure session.
        client.init_secure_channel()?;

        Ok(client)
    }

    /// Calls a contract method.
    pub fn call<Rq, Rs>(&mut self, method: &str, request: Rq) -> Result<Rs, Error>
    where
        Rq: Message,
        Rs: Message + MessageStatic,
    {
        let mut plain_request = api::PlainClientRequest::new();
        plain_request.set_method(method.to_string());
        plain_request.set_payload(request.write_to_bytes()?);

        let mut client_request = api::ClientRequest::new();
        if self.secure_channel.must_encrypt() {
            // Encrypt request.
            client_request
                .set_encrypted_request(self.secure_channel.create_request_box(&plain_request)?);
        } else {
            // Plain-text request.
            client_request.set_plain_request(plain_request);
        }

        let mut client_response = self.backend.call(client_request)?;

        if self.secure_channel.must_encrypt() && !client_response.has_encrypted_response() {
            return Err(Error::new(
                "Contract returned plain response for encrypted request",
            ));
        }

        let mut plain_response = {
            if self.secure_channel.must_encrypt() {
                // Encrypted response.
                self.secure_channel
                    .open_response_box(&client_response.get_encrypted_response())?
            } else {
                // Plain-text response.
                client_response.take_plain_response()
            }
        };

        // Validate response code.
        match plain_response.get_code() {
            api::PlainClientResponse_Code::SUCCESS => {}
            _ => {
                // Deserialize error.
                let mut error: api::Error = {
                    match protobuf::parse_from_bytes(&plain_response.take_payload()) {
                        Ok(error) => error,
                        _ => return Err(Error::new("Unknown error")),
                    }
                };

                return Err(Error::new(error.get_message()));
            }
        };

        let response: Rs = protobuf::parse_from_bytes(plain_response.get_payload())?;

        Ok(response)
    }

    /// Initialize a secure channel with the contract.
    pub fn init_secure_channel(&mut self) -> Result<(), Error> {
        let mut request = api::ChannelInitRequest::new();

        // Reset secure channel.
        self.secure_channel.reset()?;

        // Generate random nonce for contract attestation request.
        let mut nonce = vec![0u8; 16];
        random::get_random_bytes(&mut nonce)?;

        {
            // Generate contract attestation request.
            let contract_attestation = request.mut_contract_attestation_request();
            contract_attestation.set_nonce(nonce.clone());
            contract_attestation.set_spid(self.backend.get_spid()?);
        }

        request.set_short_term_public_key(self.secure_channel.get_client_public_key().to_vec());

        let mut response: api::ChannelInitResponse = self.call(api::METHOD_CHANNEL_INIT, request)?;

        // Verify quote via IAS, verify nonce.
        let quote = self.backend
            .verify_quote(response.mut_contract_attestation_response().take_quote())?;
        if quote.get_nonce().to_vec() != nonce {
            return Err(Error::new(
                "Secure channel initialization failed: nonce mismatch",
            ));
        }

        // Verify MRENCLAVE.
        if quote.get_mr_enclave() != &self.mr_enclave {
            return Err(Error::new(
                "Secure channel initialization failed: MRENCLAVE mismatch",
            ));
        }

        // TODO: Verify enclave attributes.

        // Extract public key and establish a secure channel.
        self.secure_channel
            .setup(&quote.get_public_key(), &response.get_response_box())?;

        // Check if mutual attestataion has been requested and request the backend to perform
        // client attestation. If the backend does not support this (e.g., because the client
        // is not actually running in an enclave), this operation will fail.
        if self.secure_channel.get_state() == SessionState::ClientAttestationRequired {
            let quote = self.backend.get_quote(
                &self.secure_channel.get_client_attestation_spid(),
                &self.secure_channel.get_client_attestation_nonce(),
                &self.secure_channel.get_client_public_key(),
            )?;

            let mut request = api::ChannelAttestClientRequest::new();
            request.mut_client_attestation_response().set_quote(quote);

            let _response: api::ChannelAttestClientResponse =
                self.call(api::METHOD_CHANNEL_ATTEST_CLIENT, request)?;
        }

        Ok(())
    }

    /// Close secure channel.
    pub fn close_secure_channel(&mut self) -> Result<(), Error> {
        // If secure channel is not open, do not close it.
        if self.secure_channel.get_state() == SessionState::Init {
            return Ok(());
        }

        // Send request to close channel.
        let request = api::ChannelCloseRequest::new();

        let _response: api::ChannelCloseResponse = self.call(api::METHOD_CHANNEL_CLOSE, request)?;

        // Reset local part of the secure channel.
        self.secure_channel.reset()?;

        Ok(())
    }
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

        let mut response_box: api::ChannelInitResponseBox =
            protobuf::parse_from_bytes(&response_box)?;

        self.contract_short_term_public_key
            .copy_from_slice(&response_box.get_short_term_public_key());

        // Check if client attestation is required.
        if response_box.has_client_attestation_request() {
            let mut attestation_request = response_box.take_client_attestation_request();
            self.client_attestation_nonce = attestation_request.take_nonce();
            self.client_attestation_spid = attestation_request.take_spid();
            self.state
                .transition_to(SessionState::ClientAttestationRequired)?;
        } else {
            self.state.transition_to(SessionState::Established)?;
        }

        Ok(())
    }

    /// Get secure channel session state.
    pub fn get_state(&self) -> SessionState {
        self.state
    }

    /// Check if messages must be encrypted based on current channel state.
    ///
    /// Messages can only be unencrypted when the channel is in initialization state
    /// and must be encrypted in all other states.
    pub fn must_encrypt(&self) -> bool {
        self.state != SessionState::Init
    }

    /// Get client short-term public key.
    pub fn get_client_public_key(&self) -> &sodalite::BoxPublicKey {
        &self.client_public_key
    }

    /// Get client attestation nonce for mutual attestation.
    pub fn get_client_attestation_nonce(&self) -> &Vec<u8> {
        &self.client_attestation_nonce
    }

    /// Get client attestation SPID for mutual attestation.
    pub fn get_client_attestation_spid(&self) -> &Vec<u8> {
        &self.client_attestation_spid
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
            &mut self.shared_request_key,
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
            &mut self.shared_response_key,
        )?;

        Ok(protobuf::parse_from_bytes(&plain_response)?)
    }
}

impl<Backend: ContractClientBackend> Drop for ContractClient<Backend> {
    /// Close secure channel when going out of scope.
    fn drop(&mut self) {
        self.close_secure_channel().unwrap();
    }
}
