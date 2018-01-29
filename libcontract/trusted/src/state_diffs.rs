use std;
use std::io::Write;

use bzip2;

use libcontract_common;

fn diff_internal(_old: &[u8], new: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut enc = bzip2::write::BzEncoder::new(std::io::Cursor::new(Vec::new()), bzip2::Compression::Default);
    enc.write_all(new)?;
    Ok(enc.finish()?.into_inner())
}

fn apply_internal(_old: &[u8], diff: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut dec = bzip2::write::BzDecoder::new(std::io::Cursor::new(Vec::new()));
    dec.write_all(diff)?;
    Ok(dec.finish()?.into_inner())
}

pub fn diff(
    req: &libcontract_common::api::StateDiffRequest,
) -> Result<libcontract_common::api::StateDiffResponse, libcontract_common::ContractError> {
    let old = super::state_crypto::decrypt_state(req.get_old())?;
    let new = super::state_crypto::decrypt_state(req.get_new())?;
    let diff = diff_internal(&old, &new)?;
    let mut res = libcontract_common::api::StateDiffResponse::new();
    res.set_diff(super::state_crypto::encrypt_state(diff)?);
    Ok(res)
}

pub fn apply(
    req: &libcontract_common::api::StateApplyRequest,
) -> Result<libcontract_common::api::StateApplyResponse, libcontract_common::ContractError> {
    let old = super::state_crypto::decrypt_state(req.get_old())?;
    let diff = super::state_crypto::decrypt_state(req.get_diff())?;
    let new = apply_internal(&old, &diff)?;
    let mut res = libcontract_common::api::StateApplyResponse::new();
    res.set_new(super::state_crypto::encrypt_state(new)?);
    Ok(res)
}
