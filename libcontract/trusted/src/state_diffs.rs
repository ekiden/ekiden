use libcontract_common;

pub fn diff_internal(_old: &[u8], new: &[u8]) -> Vec<u8> {
    new.to_vec()
}

pub fn apply_internal(_old: &[u8], diff: &[u8]) -> Vec<u8> {
    diff.to_vec()
}

pub fn diff(req: &libcontract_common::api::StateDiffRequest) -> Result<libcontract_common::api::StateDiffResponse, libcontract_common::ContractError> {
    let old = super::state_crypto::decrypt_state(req.get_old())?;
    let new = super::state_crypto::decrypt_state(req.get_new())?;
    let diff = diff_internal(&old, &new);
    let mut res = libcontract_common::api::StateDiffResponse::new();
    res.set_diff(super::state_crypto::encrypt_state(diff)?);
    Ok(res)
}

pub fn apply(req: &libcontract_common::api::StateApplyRequest) -> Result<libcontract_common::api::StateApplyResponse, libcontract_common::ContractError> {
    let old = super::state_crypto::decrypt_state(req.get_old())?;
    let diff = super::state_crypto::decrypt_state(req.get_diff())?;
    let new = diff_internal(&old, &diff);
    let mut res = libcontract_common::api::StateApplyResponse::new();
    res.set_new(super::state_crypto::encrypt_state(new)?);
    Ok(res)
}
