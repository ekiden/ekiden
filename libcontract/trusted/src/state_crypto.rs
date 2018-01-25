use protobuf;
use sodalite;

use libcontract_common::{random, ContractError};
use libcontract_common::api::CryptoSecretbox;

use super::key_manager::KeyManager;

const SECRETBOX_ZEROBYTES: usize = 32;

/// Retrieve or generate state secret key.
fn get_state_key() -> Result<sodalite::SecretboxKey, ContractError> {
    if KeyManager::is_self() {
        // State for the key manager itself.
        Err(ContractError::new(
            "State for key manager not yet implemented",
        ))
    } else {
        let key = KeyManager::get()?.get_or_create_key("state", sodalite::SECRETBOX_KEY_LEN)?;
        let mut state_key = [0; sodalite::SECRETBOX_KEY_LEN];
        state_key.copy_from_slice(key.as_slice());

        Ok(state_key)
    }
}

/// Open encrypted state box.
pub fn decrypt_state<S: protobuf::MessageStatic>(
    encrypted_state: &CryptoSecretbox,
) -> Result<S, ContractError> {
    let state_key = get_state_key()?;
    let encrypted_state_ciphertext = encrypted_state.get_ciphertext();

    let mut encrypted_state_nonce: sodalite::SecretboxNonce = [0; sodalite::SECRETBOX_NONCE_LEN];
    encrypted_state_nonce.copy_from_slice(encrypted_state.get_nonce());

    let mut state_raw_padded = vec![0; encrypted_state_ciphertext.len()];

    match sodalite::secretbox_open(
        &mut state_raw_padded,
        encrypted_state_ciphertext,
        &encrypted_state_nonce,
        &state_key,
    ) {
        Ok(_) => {}
        _ => return Err(ContractError::new("Failed to open state box")),
    }

    Ok(protobuf::parse_from_bytes(
        &state_raw_padded[SECRETBOX_ZEROBYTES..],
    )?)
}

/// Generate encrypted state box.
pub fn encrypt_state<S: protobuf::Message>(state: &S) -> Result<CryptoSecretbox, ContractError> {
    let state_key = get_state_key()?;

    let mut state_raw_padded = vec![0; SECRETBOX_ZEROBYTES];
    state.write_to_vec(&mut state_raw_padded)?;

    let mut encrypted_state_nonce = [0; sodalite::SECRETBOX_NONCE_LEN];
    random::get_random_bytes(&mut encrypted_state_nonce)?;

    let mut encrypted_state_ciphertext = vec![0; state_raw_padded.len()];

    match sodalite::secretbox(
        &mut encrypted_state_ciphertext,
        &state_raw_padded,
        &encrypted_state_nonce,
        &state_key,
    ) {
        Ok(_) => {}
        _ => return Err(ContractError::new("Failed to create state box")),
    }

    let mut encrypted_state = CryptoSecretbox::new();
    encrypted_state.set_ciphertext(encrypted_state_ciphertext);
    encrypted_state.set_nonce(encrypted_state_nonce.to_vec());

    Ok(encrypted_state)
}
