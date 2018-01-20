#[cfg(not(feature = "sgx"))]
use rand::{OsRng, Rng};

#[cfg(feature = "sgx")]
use sgx_trts;

use super::ContractError;

/// Fill destination type with random bytes.
#[cfg(feature = "sgx")]
pub fn get_random_bytes(destination: &mut [u8]) -> Result<(), ContractError> {
    match sgx_trts::rsgx_read_rand(destination) {
        Ok(_) => {}
        _ => return Err(ContractError::new("Random bytes failed")),
    }

    Ok(())
}

/// Fill destination type with random bytes.
#[cfg(not(feature = "sgx"))]
pub fn get_random_bytes(destination: &mut [u8]) -> Result<(), ContractError> {
    let mut rng = match OsRng::new() {
        Ok(rng) => rng,
        _ => return Err(ContractError::new("Random bytes failed")),
    };

    rng.fill_bytes(destination);

    Ok(())
}
