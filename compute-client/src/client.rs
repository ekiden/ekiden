use sodalite;

use protobuf;
use protobuf::{Message, MessageStatic};

use libcontract_common::{api, random};
use libcontract_common::secure_channel::{create_box, open_box, RandomNonceGenerator, MonotonicNonceGenerator,
                                         NONCE_CONTEXT_INIT, NONCE_CONTEXT_REQUEST, NONCE_CONTEXT_RESPONSE};

use super::errors::Error;
use super::backend::ContractClientBackend;
use super::quote::MrEnclave;

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
    /// Channel status.
    ready: bool,
    /// Long-term nonce generator.
    long_term_nonce_generator: RandomNonceGenerator,
    /// Short-term nonce generator.
    short_term_nonce_generator: MonotonicNonceGenerator,
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
    pub fn new(backend: Backend,
               mr_enclave: MrEnclave) -> Result<Self, Error> {

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
        where Rq: Message,
              Rs: Message + MessageStatic {

        let mut plain_request = api::PlainClientRequest::new();
        plain_request.set_method(method.to_string());
        plain_request.set_payload(request.write_to_bytes()?);

        let mut client_request = api::ClientRequest::new();
        if self.secure_channel.ready {
            // Encrypt request.
            client_request.set_encrypted_request(
                self.secure_channel.create_request_box(&plain_request)?
            );
        } else {
            // Plain-text request.
            client_request.set_plain_request(plain_request);
        }

        let mut client_response = self.backend.call(client_request)?;

        if self.secure_channel.ready && !client_response.has_encrypted_response() {
            return Err(Error::new("Contract returned plain response for encrypted request"))
        }

        let mut plain_response = {
            if self.secure_channel.ready {
                // Encrypted response.
                self.secure_channel.open_response_box(&client_response.get_encrypted_response())?
            } else {
                // Plain-text response.
                client_response.take_plain_response()
            }
        };

        // Validate response code.
        match plain_response.get_code() {
            api::PlainClientResponse_Code::SUCCESS => {},
            _ => {
                // Deserialize error.
                let mut error: api::Error = match protobuf::parse_from_bytes(&plain_response.take_payload()) {
                    Ok(error) => error,
                    _ => return Err(Error::new("Unknown error"))
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

        // Generate random nonce.
        let mut nonce = vec![0u8; 16];
        random::get_random_bytes(&mut nonce)?;

        request.set_nonce(nonce.clone());
        request.set_spid(self.backend.get_spid()?);
        request.set_short_term_public_key(self.secure_channel.get_client_public_key().to_vec());

        let mut response: api::ChannelInitResponse = self.call("_channel_init", request)?;

        // Verify quote via IAS, verify nonce.
        let quote = self.backend.verify_quote(response.take_quote())?;
        if quote.get_nonce().to_vec() != nonce {
            return Err(Error::new("Secure channel initialization failed: nonce mismatch"));
        }

        // Verify MRENCLAVE.
        if quote.get_mr_enclave() != &self.mr_enclave {
            return Err(Error::new("Secure channel initialization failed: MRENCLAVE mismatch"));
        }

        // Extract public key and establish a secure channel.
        self.secure_channel.setup(
            &quote.get_public_key(),
            &response.get_short_term_public_key(),
        )?;

        Ok(())
    }

    /// Close secure channel.
    pub fn close_secure_channel(&mut self) -> Result<(), Error> {
        // Send request to close channel.
        let request = api::ChannelCloseRequest::new();

        let _response: api::ChannelCloseResponse = self.call("_channel_close", request)?;

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
            &seed
        );

        // Clear contract keys.
        self.contract_long_term_public_key = [0; sodalite::BOX_PUBLIC_KEY_LEN];
        self.contract_short_term_public_key = [0; sodalite::BOX_PUBLIC_KEY_LEN];
        self.ready = false;

        Ok(())
    }

    /// Setup secure channel.
    pub fn setup(&mut self,
                 contract_long_term_public_key: &[u8],
                 contract_short_term_public_key: &api::CryptoBox) -> Result<(), Error> {

        self.contract_long_term_public_key.copy_from_slice(&contract_long_term_public_key);

        // Open boxed short term server public key.
        let mut shared_key: Option<sodalite::SecretboxKey> = None;
        let contract_short_term_public_key = open_box(
            &contract_short_term_public_key,
            &NONCE_CONTEXT_INIT,
            &mut self.long_term_nonce_generator,
            &self.contract_long_term_public_key,
            &self.client_private_key,
            &mut shared_key,
        )?;

        self.contract_short_term_public_key.copy_from_slice(&contract_short_term_public_key);
        self.ready = true;

        Ok(())
    }

    /// Get client short-term public key.
    pub fn get_client_public_key(&self) -> &sodalite::BoxPublicKey {
        &self.client_public_key
    }

    /// Create cryptographic box with RPC request.
    pub fn create_request_box(&mut self, request: &api::PlainClientRequest) -> Result<api::CryptoBox, Error> {
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
    pub fn open_response_box(&mut self, response: &api::CryptoBox) -> Result<api::PlainClientResponse, Error> {
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
