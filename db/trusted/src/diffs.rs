use std;

use bsdiff;
use bzip2;
use protobuf;
use protobuf::Message;

use ekiden_common::error::Result;

use super::crypto;
use super::generated::database::BsdiffPatch;

/// Diff: create a summary of changes that can be applied to `old` to recreate `new`.
/// This is the actual diffing algorithm implementation.
fn diff_internal(old: &[u8], new: &[u8]) -> std::io::Result<Vec<u8>> {
    let mut enc = bzip2::write::BzEncoder::new(
        std::io::Cursor::new(Vec::new()),
        bzip2::Compression::Default,
    );
    bsdiff::diff::diff(old, new, &mut enc)?;
    let mut m = BsdiffPatch::new();
    m.set_new_length(new.len() as u64);
    m.set_patch_bz2(enc.finish()?.into_inner());
    Ok(m.write_to_bytes()?)
}

/// Apply: change `old` as specified by `diff`.
/// `apply_internal(&old, &diff_internal(&old, &new))` should be the same as `new`.
fn apply_internal(old: &[u8], diff: &[u8]) -> std::io::Result<Vec<u8>> {
    let m: BsdiffPatch = protobuf::parse_from_bytes(diff)?;
    let mut dec = bzip2::read::BzDecoder::new(std::io::Cursor::new(m.get_patch_bz2()));
    let mut new = vec![0; m.get_new_length() as usize];
    bsdiff::patch::patch(old, &mut dec, &mut new)?;
    Ok(new)
}

pub fn diff(old: &Vec<u8>, new: &Vec<u8>) -> Result<Vec<u8>> {
    let old = crypto::decrypt_state(&protobuf::parse_from_bytes(&old)?)?;
    let new = crypto::decrypt_state(&protobuf::parse_from_bytes(&new)?)?;
    let diff = diff_internal(&old, &new)?;

    Ok(crypto::encrypt_state(diff)?.write_to_bytes()?)
}

pub fn apply(old: &Vec<u8>, diff: &Vec<u8>) -> Result<Vec<u8>> {
    let old = crypto::decrypt_state(&protobuf::parse_from_bytes(old)?)?;
    let diff = crypto::decrypt_state(&protobuf::parse_from_bytes(diff)?)?;
    let new = apply_internal(&old, &diff)?;

    Ok(crypto::encrypt_state(new)?.write_to_bytes()?)
}
