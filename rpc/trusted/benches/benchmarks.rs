#![feature(test)]

extern crate protobuf;
extern crate sodalite;
extern crate test;

extern crate ekiden_common;
extern crate ekiden_rpc_common;
extern crate ekiden_rpc_trusted;

use test::Bencher;

use protobuf::{Message, MessageStatic};
use protobuf::well_known_types::Empty;

use ekiden_common::error::Result;
use ekiden_common::random;
use ekiden_rpc_common::api;
use ekiden_rpc_common::reflection::ApiMethodDescriptor;
use ekiden_rpc_common::secure_channel::{create_box, open_box, MonotonicNonceGenerator,
                                        RandomNonceGenerator, NONCE_CONTEXT_INIT,
                                        NONCE_CONTEXT_REQUEST, NONCE_CONTEXT_RESPONSE};
use ekiden_rpc_trusted::dispatcher::{rpc_call, Dispatcher, EnclaveMethod};
use ekiden_rpc_trusted::request::Request;
use ekiden_rpc_trusted::secure_channel::SecureChannelContext;

/// Register an empty method.
fn register_empty_method() {
    let mut dispatcher = Dispatcher::get();

    // Register dummy RPC method.
    dispatcher.add_method(EnclaveMethod::new(
        ApiMethodDescriptor {
            name: "benchmark_empty".to_owned(),
            client_attestation_required: false,
        },
        |_request: &Request<Empty>| -> Result<Empty> { Ok(Empty::new()) },
    ));
}

/// Prepare secure channel enclave parameters.
fn prepare_secure_channel_enclave() {
    let mut ctx = SecureChannelContext::get();

    if !ctx.is_ready() {
        ctx.generate_keypair().unwrap();
    }
}

/// Prepare secure channel client parameters.
fn prepare_secure_channel_client() -> (sodalite::BoxPublicKey, sodalite::BoxSecretKey) {
    // Generate new short-term key pair for the client.
    let mut seed = [0u8; 32];
    random::get_random_bytes(&mut seed).unwrap();

    let mut public_key: sodalite::BoxPublicKey = [0u8; 32];
    let mut private_key: sodalite::BoxSecretKey = [0u8; 32];
    sodalite::box_keypair_seed(&mut public_key, &mut private_key, &seed);

    (public_key, private_key)
}

/// Dispatch secure channel initialization request.
fn init_secure_channel(
    public_key: &sodalite::BoxPublicKey,
    private_key: &sodalite::BoxSecretKey,
) -> sodalite::BoxPublicKey {
    // Generate request.
    let mut request = api::ChannelInitRequest::new();
    request.set_short_term_public_key(public_key.to_vec());

    // Dispatch channel init request.
    let request = Request::new(
        request.write_to_bytes().unwrap(),
        api::METHOD_CHANNEL_INIT.to_owned(),
        None,
        None,
    );

    let dispatcher = Dispatcher::get();
    let mut response = dispatcher.dispatch(request);
    let response = response.take_message().take_plain_response();
    assert_eq!(response.get_code(), api::PlainClientResponse_Code::SUCCESS);

    let response: api::ChannelInitResponse =
        protobuf::parse_from_bytes(response.get_payload()).unwrap();

    let ctx = SecureChannelContext::get();
    let contract_long_term_public_key = ctx.get_public_key();
    let mut nonce_generator = RandomNonceGenerator::new();
    let mut shared_key: Option<sodalite::SecretboxKey> = None;
    let response_box = open_box(
        &response.get_response_box(),
        &NONCE_CONTEXT_INIT,
        &mut nonce_generator,
        &contract_long_term_public_key,
        &private_key,
        &mut shared_key,
    ).unwrap();

    let response_box: api::ChannelInitResponseBox =
        protobuf::parse_from_bytes(&response_box).unwrap();

    let mut short_term_public_key = [0u8; 32];
    short_term_public_key.copy_from_slice(&response_box.get_short_term_public_key());

    short_term_public_key
}

/// Dispatch secure channel request.
fn make_secure_channel_request<S, Rq, Rs>(
    nonce_generator: &mut MonotonicNonceGenerator,
    contract_public_key: &sodalite::BoxPublicKey,
    public_key: &sodalite::BoxPublicKey,
    private_key: &sodalite::BoxSecretKey,
    mut shared_key: &mut Option<sodalite::SecretboxKey>,
    method: S,
    request: Rq,
) -> Rs
where
    S: Into<String>,
    Rq: Message,
    Rs: Message + MessageStatic,
{
    let mut plain_client_request = api::PlainClientRequest::new();
    plain_client_request.set_method(method.into());
    plain_client_request.set_payload(request.write_to_bytes().unwrap());

    let mut crypto_box = create_box(
        &plain_client_request.write_to_bytes().unwrap(),
        &NONCE_CONTEXT_REQUEST,
        nonce_generator,
        &contract_public_key,
        &private_key,
        &mut shared_key,
    ).unwrap();

    // Set public key so the contract knows which client this is.
    crypto_box.set_public_key(public_key.to_vec());

    let mut client_request = api::ClientRequest::new();
    client_request.set_encrypted_request(crypto_box);

    // Generate encrypted enclave request.
    let mut enclave_request = api::EnclaveRequest::new();
    enclave_request.mut_client_request().push(client_request);

    let enclave_request = enclave_request.write_to_bytes().unwrap();

    let mut response: Vec<u8> = Vec::with_capacity(64 * 1024);
    let mut response_length = 0;

    // Invoke the RPC call ECALL handler.
    rpc_call(
        enclave_request.as_ptr(),
        enclave_request.len(),
        response.as_mut_ptr(),
        response.capacity(),
        &mut response_length,
    );

    unsafe {
        response.set_len(response_length);
    }

    // Decrypt response.
    let enclave_response: api::EnclaveResponse = protobuf::parse_from_bytes(&response).unwrap();
    assert_eq!(enclave_response.get_client_response().len(), 1);

    let client_response = &enclave_response.get_client_response()[0];
    assert!(client_response.has_encrypted_response());

    let plain_response = open_box(
        &client_response.get_encrypted_response(),
        &NONCE_CONTEXT_RESPONSE,
        nonce_generator,
        &contract_public_key,
        &private_key,
        &mut shared_key,
    ).unwrap();

    let plain_response: api::PlainClientResponse =
        protobuf::parse_from_bytes(&plain_response).unwrap();
    assert_eq!(
        plain_response.get_code(),
        api::PlainClientResponse_Code::SUCCESS
    );

    protobuf::parse_from_bytes(&plain_response.get_payload()).unwrap()
}

/// Benchmark dispatch of a plain empty Protocol Buffers request.
#[bench]
fn benchmark_dispatch_empty_request(b: &mut Bencher) {
    register_empty_method();

    // Prepare a dummy request.
    let request = Request::new(
        Empty::new().write_to_bytes().unwrap(),
        "benchmark_empty".to_owned(),
        None,
        None,
    );

    b.iter(|| {
        let dispatcher = Dispatcher::get();
        let mut response = dispatcher.dispatch(request.clone());
        assert_eq!(
            response.take_message().get_plain_response().get_code(),
            api::PlainClientResponse_Code::SUCCESS
        );
    });
}

/// Benchmark secure channel initialization.
///
/// Note that this includes generating client cryptographic parameters.
#[bench]
fn benchmark_secure_channel_init(b: &mut Bencher) {
    register_empty_method();
    prepare_secure_channel_enclave();

    b.iter(|| {
        let (public_key, private_key) = prepare_secure_channel_client();
        init_secure_channel(&public_key, &private_key);
    });
}

/// Benchmark dispatch of an encrypted empty Protocol Buffers request over a secure channel,
/// where the shared key is only derived once and then cached (which is what the actual
/// client does as well).
///
/// Note that this includes generating encrypted requests for the client.
#[bench]
fn benchmark_secure_channel_empty_request(b: &mut Bencher) {
    register_empty_method();
    prepare_secure_channel_enclave();
    let (public_key, private_key) = prepare_secure_channel_client();
    let contract_public_key = init_secure_channel(&public_key, &private_key);
    let mut nonce_generator = MonotonicNonceGenerator::new();
    let mut shared_key: Option<sodalite::SecretboxKey> = None;

    // First request to initialize shared key.
    let _response: Empty = make_secure_channel_request(
        &mut nonce_generator,
        &contract_public_key,
        &public_key,
        &private_key,
        &mut shared_key,
        "benchmark_empty",
        Empty::new(),
    );

    b.iter(|| {
        let _response: Empty = make_secure_channel_request(
            &mut nonce_generator,
            &contract_public_key,
            &public_key,
            &private_key,
            &mut shared_key,
            "benchmark_empty",
            Empty::new(),
        );
    });
}

/// Benchmark dispatch of an encrypted empty Protocol Buffers request over a secure channel,
/// where the shared key is derived each time, requring the use of expensive public key
/// operations.
///
/// Note that this includes generating encrypted requests for the client.
#[bench]
fn benchmark_secure_channel_empty_request_no_shared_key(b: &mut Bencher) {
    register_empty_method();
    prepare_secure_channel_enclave();
    let (public_key, private_key) = prepare_secure_channel_client();
    let contract_public_key = init_secure_channel(&public_key, &private_key);
    let mut nonce_generator = MonotonicNonceGenerator::new();

    b.iter(|| {
        // Use an empty shared key each time to force expensive public key ops.
        let mut shared_key: Option<sodalite::SecretboxKey> = None;
        let _response: Empty = make_secure_channel_request(
            &mut nonce_generator,
            &contract_public_key,
            &public_key,
            &private_key,
            &mut shared_key,
            "benchmark_empty",
            Empty::new(),
        );
    });
}
