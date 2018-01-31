use std;

use bsdiff;
use bzip2;
use protobuf;
use protobuf::Message;

use libcontract_common;

/// Diff: create a summary of changes that can be applied to `old` to recreate `new`.
/// This is the actual diffing algorithm implementation.
fn diff_internal(old: &[u8], new: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut enc = bzip2::write::BzEncoder::new(
        std::io::Cursor::new(Vec::new()),
        bzip2::Compression::Default,
    );
    bsdiff::diff::diff(old, new, &mut enc)?;
    let mut m = libcontract_common::api::BsdiffPatch::new();
    m.set_new_length(new.len() as u64);
    m.set_patch_bz2(enc.finish()?.into_inner());
    Ok(m.write_to_bytes()?)
}

/// Apply: change `old` as specified by `diff`.
/// `apply_internal(&old, &diff_internal(&old, &new))` should be the same as `new`.
fn apply_internal(old: &[u8], diff: &[u8]) -> std::io::Result<Vec<u8>> {
    let m: libcontract_common::api::BsdiffPatch = protobuf::parse_from_bytes(diff)?;
    let mut dec = bzip2::read::BzDecoder::new(std::io::Cursor::new(m.get_patch_bz2()));
    let mut new = vec![0; m.get_new_length() as usize];
    bsdiff::patch::patch(old, &mut dec, &mut new)?;
    Ok(new)
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
