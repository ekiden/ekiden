use sgx_trts;

use protobuf;
use sodalite;

use libcontract_common::api::CryptoSecretbox;

const SECRETBOX_ZEROBYTES: usize = 32;

fn get_state_key() -> sodalite::SecretboxKey {
    // TODO: load from sealed
    let mut dummy_key = [0; sodalite::SECRETBOX_KEY_LEN];
    for i in 0..dummy_key.len() {
        dummy_key[i] = i as u8;
    }
    dummy_key
}

pub fn decrypt_state<S: protobuf::MessageStatic>(
    encrypted_state: &CryptoSecretbox,
) -> Result<S, protobuf::ProtobufError> {
    let state_key = get_state_key();
    let encrypted_state_ciphertext = encrypted_state.get_ciphertext();
    let mut encrypted_state_nonce: sodalite::SecretboxNonce = [0; sodalite::SECRETBOX_NONCE_LEN];
    encrypted_state_nonce.copy_from_slice(encrypted_state.get_nonce());
    let mut state_raw_padded = vec![0; encrypted_state_ciphertext.len()];
    // TODO: propagate errors from here?
    sodalite::secretbox_open(
        &mut state_raw_padded,
        encrypted_state_ciphertext,
        &encrypted_state_nonce,
        &state_key,
    ).unwrap();
    Ok(protobuf::parse_from_bytes(
        &state_raw_padded[SECRETBOX_ZEROBYTES..],
    )?)
}

pub fn encrypt_state<S: protobuf::Message>(
    state: &S,
) -> Result<CryptoSecretbox, protobuf::ProtobufError> {
    let state_key = super::state_crypto::get_state_key();
    let mut state_raw_padded = vec![0; super::state_crypto::SECRETBOX_ZEROBYTES];
    state.write_to_vec(&mut state_raw_padded)?;
    let mut encrypted_state_nonce = [0; sodalite::SECRETBOX_NONCE_LEN];
    // TODO: propagate errors from here?
    sgx_trts::rsgx_read_rand(&mut encrypted_state_nonce).unwrap();
    let mut encrypted_state_ciphertext = vec![0; state_raw_padded.len()];
    // TODO: propagate errors from here?
    sodalite::secretbox(
        &mut encrypted_state_ciphertext,
        &state_raw_padded,
        &encrypted_state_nonce,
        &state_key,
    ).unwrap();
    let mut encrypted_state = CryptoSecretbox::new();
    encrypted_state.set_ciphertext(encrypted_state_ciphertext);
    encrypted_state.set_nonce(encrypted_state_nonce.to_vec());
    Ok(encrypted_state)
}
