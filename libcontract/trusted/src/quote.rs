use sgx_trts;
use sgx_tse;
use sgx_types::*;

use sodalite;

use libcontract_common::{api, random, ContractError};
use libcontract_common::client::ClientEndpoint;
use libcontract_common::quote::*;

use super::dispatcher;
use super::untrusted;

pub const REPORT_DATA_LEN: usize = SGX_REPORT_DATA_SIZE - QUOTE_CONTEXT_LEN;
pub type ReportData = [u8; REPORT_DATA_LEN];

/// Internal helper macro for SGX OCALLs.
macro_rules! sgx_call {
    ($error: expr, $result: ident, $block: block) => {
        let status = unsafe { $block };

        match status {
            sgx_status_t::SGX_SUCCESS => {
                match $result {
                    sgx_status_t::SGX_SUCCESS => {},
                    _ => return Err(ContractError::new($error))
                };
            },
            _ => return Err(ContractError::new($error))
        };
    }
}

/// Create report containg a public key and a nonce.
///
/// This type of report is used when creating quotes for attestation of
/// Ekiden enclaves.
pub fn create_report_data_for_public_key(
    nonce: &[u8],
    public_key: &sodalite::BoxPublicKey,
) -> Result<ReportData, ContractError> {
    if nonce.len() != 16 {
        return Err(ContractError::new("Invalid nonce"));
    }

    let mut report_data: ReportData = [0; REPORT_DATA_LEN];
    let pkey_len = sodalite::BOX_PUBLIC_KEY_LEN;
    report_data[..pkey_len].copy_from_slice(public_key);
    report_data[pkey_len..pkey_len + 16].copy_from_slice(nonce);

    Ok(report_data)
}

/// Generate a quote suitable for remote attestation.
///
/// The `spid` parameter should be an IAS SPID that can be used by the remote
/// party to verify this quote. Arbitrary data can be included in the quote
/// using `report_data`.
///
/// The purpose of `quote_context` is to prevent quotes from being used in
/// different contexts. The value is included as a prefix in report data.
pub fn get_quote(
    spid: &[u8],
    quote_context: &QuoteContext,
    report_data: ReportData,
) -> Result<Vec<u8>, ContractError> {
    if spid.len() != 16 {
        return Err(ContractError::new("Invalid SPID"));
    }

    // Initialize target suitable for use by the quoting enclave.
    let mut result = sgx_status_t::SGX_ERROR_UNEXPECTED;
    let mut target_info = sgx_target_info_t::default();
    let mut epid_group = sgx_epid_group_id_t::default();

    sgx_call!("Failed to initialize quote", result, {
        untrusted::untrusted_init_quote(
            &mut result,
            &mut target_info as *mut sgx_target_info_t,
            &mut epid_group as *mut sgx_epid_group_id_t,
        )
    });

    // Generate report for the quoting enclave (include channel public key in report data).
    let mut context_report_data: [u8; SGX_REPORT_DATA_SIZE] = [0; SGX_REPORT_DATA_SIZE];
    context_report_data[..QUOTE_CONTEXT_LEN].copy_from_slice(&quote_context[..QUOTE_CONTEXT_LEN]);
    context_report_data[QUOTE_CONTEXT_LEN..].copy_from_slice(&report_data[..REPORT_DATA_LEN]);

    let report_data = sgx_report_data_t {
        d: context_report_data,
    };
    let report = match sgx_tse::rsgx_create_report(&target_info, &report_data) {
        Ok(report) => report,
        _ => return Err(ContractError::new("Failed to create report")),
    };

    // Request the quoting enclave to generate a quote from our report.
    let mut qe_report = sgx_report_t::default();
    let mut qe_nonce = sgx_quote_nonce_t { rand: [0; 16] };
    let mut s_spid = sgx_spid_t { id: [0; 16] };

    // Maximum quote size is 16K.
    let mut quote: Vec<u8> = Vec::with_capacity(16 * 1024);
    let mut quote_size = 0;

    s_spid.id.copy_from_slice(&spid[..16]);

    match sgx_trts::rsgx_read_rand(&mut qe_nonce.rand) {
        Ok(_) => {}
        _ => return Err(ContractError::new("Failed to generate random nonce")),
    };

    sgx_call!("Failed to get quote", result, {
        untrusted::untrusted_get_quote(
            &mut result,
            &report as *const sgx_report_t,
            sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
            &s_spid as *const sgx_spid_t,
            &qe_nonce as *const sgx_quote_nonce_t,
            &mut qe_report as *mut sgx_report_t,
            quote.as_mut_ptr() as *mut u8,
            quote.capacity() as u32,
            &mut quote_size,
        )
    });

    match sgx_tse::rsgx_verify_report(&qe_report) {
        Ok(_) => {}
        _ => return Err(ContractError::new("Failed to get quote")),
    };

    unsafe {
        quote.set_len(quote_size as usize);
    }

    // TODO: Verify QE signature. Note that this may not be the QE enclave at all as
    // untrusted_init_quote can provide an arbitrary enclave target. Is there a way
    // to get the QE identity in a secure way?
    // lower 32Bytes in report.data = SHA256(qe_nonce||quote).

    Ok(quote)
}

/// Get SPID that can be used to verify the quote later.
pub fn get_spid() -> Result<Vec<u8>, ContractError> {
    let request = api::services::IasGetSpidRequest::new();
    let mut response: api::services::IasGetSpidResponse =
        dispatcher::untrusted_call_endpoint(&ClientEndpoint::IASProxyGetSpid, request)?;

    Ok(response.take_spid())
}

/// Verify quote via IAS.
///
/// The quote must have been generated using an SPID returned by `get_spid`.
pub fn verify_quote(quote: Vec<u8>) -> Result<Quote, ContractError> {
    let decoded = Quote::decode(&quote)?;

    let mut request = api::services::IasVerifyQuoteRequest::new();
    request.set_quote(quote);

    // Generate random nonce.
    let mut nonce = vec![0u8; 16];
    random::get_random_bytes(&mut nonce)?;
    request.set_nonce(nonce.clone());

    let response: api::services::IasVerifyQuoteResponse =
        dispatcher::untrusted_call_endpoint(&ClientEndpoint::IASProxyVerifyQuote, request)?;

    // TODO: Check response, verify signatures, verify nonce etc.

    Ok(decoded)
}
