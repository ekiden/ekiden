use rand::{OsRng, Rng};

use grpc;
use sodalite;

use protobuf;
use protobuf::{Message, MessageStatic};

use libcontract_common::api::{Request, PlainRequest, Response, PlainResponse, PlainResponse_Code,
                              Error as ResponseError, ChannelInitRequest, ChannelInitResponse, CryptoBox,
                              ChannelCloseRequest, ChannelCloseResponse};
use libcontract_common::secure_channel::{create_box, open_box, RandomNonceGenerator, MonotonicNonceGenerator,
                                         NONCE_CONTEXT_INIT, NONCE_CONTEXT_REQUEST, NONCE_CONTEXT_RESPONSE};

use super::errors::Error;
use super::generated::compute_web3::{StatusRequest, CallContractRequest};
use super::generated::compute_web3_grpc::{Compute, ComputeClient};
use super::ias::{IAS, IASConfiguration, MrEnclave};

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
pub struct ContractClient {
    /// gRPC client instance.
    client: ComputeClient,
    /// IAS interface instance.
    ias: IAS,
    /// Contract MRENCLAVE.
    mr_enclave: MrEnclave,
    /// Secure channel context.
    secure_channel: SecureChannelContext,
}

pub struct ContractStatus {
    /// Contract name.
    pub contract: String,
    /// Contract version.
    pub version: String,
}

impl ContractClient {
    /// Constructs a new contract client.
    pub fn new(host: &str,
               port: u16,
               mr_enclave: MrEnclave,
               ias_config: Option<IASConfiguration>) -> Result<Self, Error> {

        Ok(ContractClient {
            // TODO: Use TLS client.
            client: ComputeClient::new_plain(&host, port, Default::default()).unwrap(),
            mr_enclave: mr_enclave,
            ias: IAS::new(ias_config)?,
            secure_channel: SecureChannelContext::default(),
        })
    }

    /// Calls a contract method.
    // TODO: have the compute node fetch and store the state
    pub fn call<Rq, Rs>(&mut self, method: &str, state: Vec<u8>, request: Rq) -> Result<(Vec<u8>, Rs), Error>
        where Rq: Message,
              Rs: Message + MessageStatic {

        let mut plain_request = PlainRequest::new();
        plain_request.set_method(method.to_string());
        plain_request.set_payload(request.write_to_bytes()?);

        let mut enclave_request = Request::new();
        if self.secure_channel.ready {
            // Encrypt request.
            enclave_request.set_encrypted_request(
                self.secure_channel.create_request_box(&plain_request)?
            );
        } else {
            // Plain-text request.
            enclave_request.set_plain_request(plain_request);
        }

        let mut raw_request = CallContractRequest::new();
        raw_request.set_method(method.to_string());
        raw_request.set_payload(request.write_to_bytes().unwrap());
        raw_request.set_payload(enclave_request.write_to_bytes()?);
        raw_request.set_encrypted_state(state);

        let (_, response, _) = self.client.call_contract(
            grpc::RequestOptions::new(),
            raw_request
        ).wait().unwrap();

        let new_state = response.get_encrypted_state().to_vec();
        let mut response: Response = protobuf::parse_from_bytes(response.get_payload())?;
        if self.secure_channel.ready && !response.has_encrypted_response() {
            return Err(Error::new("Contract returned plain response for encrypted request"))
        }

        let mut plain_response = {
            if self.secure_channel.ready {
                // Encrypted response.
                self.secure_channel.open_response_box(&response.get_encrypted_response())?
            } else {
                // Plain-text response.
                response.take_plain_response()
            }
        };

        // Validate response code.
        match plain_response.get_code() {
            PlainResponse_Code::SUCCESS => {},
            _ => {
                // Deserialize error.
                let mut error: ResponseError = match protobuf::parse_from_bytes(&plain_response.take_payload()) {
                    Ok(error) => error,
                    _ => return Err(Error::new("Unknown error"))
                };

                return Err(Error::new(error.get_message()));
            }
        };

        let response: Rs = protobuf::parse_from_bytes(plain_response.get_payload())?;

        Ok((new_state, response))
    }

    /// Get compute node status.
    pub fn status(&self) -> Result<ContractStatus, Error> {
        let request = StatusRequest::new();
        let (_, mut response, _) = self.client.status(grpc::RequestOptions::new(), request).wait().unwrap();

        let mut contract = response.take_contract();

        Ok(ContractStatus {
            contract: contract.take_name(),
            version: contract.take_version(),
        })
    }

    /// Initialize a secure channel with the contract.
    pub fn init_secure_channel(&mut self) -> Result<(), Error> {
        let mut request = ChannelInitRequest::new();

        // Reset secure channel.
        self.secure_channel.reset()?;

        // Generate random nonce.
        let mut nonce = vec![0u8; 16];
        OsRng::new()?.fill_bytes(&mut nonce);

        request.set_nonce(nonce.clone());
        request.set_spid(self.ias.get_spid().to_vec());
        request.set_short_term_public_key(self.secure_channel.get_client_public_key().to_vec());

        let mut response: ChannelInitResponse = self.call("_channel_init", request)?;

        // Verify quote via IAS, verify nonce.
        let quote = self.ias.verify_quote(&response.take_quote())?;
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
        let request = ChannelCloseRequest::new();

        let _response: ChannelCloseResponse = self.call("_channel_close", request)?;

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
        OsRng::new()?.fill_bytes(&mut seed);

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
                 contract_short_term_public_key: &CryptoBox) -> Result<(), Error> {

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
    pub fn create_request_box(&mut self, request: &PlainRequest) -> Result<CryptoBox, Error> {
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
    pub fn open_response_box(&mut self, response: &CryptoBox) -> Result<PlainResponse, Error> {
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
