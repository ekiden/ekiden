use sgx_tse;
use sgx_types::*;

use sodalite;

use ekiden_common::error::{Error, Result};
use ekiden_common::random;
use ekiden_enclave_common::quote::*;
use ekiden_rpc_common::api;
use ekiden_rpc_common::client::ClientEndpoint;

use super::bridge;
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
                    _ => return Err(Error::new($error))
                };
            },
            _ => return Err(Error::new($error))
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
) -> Result<ReportData> {
    if nonce.len() != 16 {
        return Err(Error::new("Invalid nonce"));
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
) -> Result<Vec<u8>> {
    if spid.len() != 16 {
        return Err(Error::new("Invalid SPID"));
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
        _ => return Err(Error::new("Failed to create report")),
    };

    // Request the quoting enclave to generate a quote from our report.
    let mut s_spid = sgx_spid_t { id: [0; 16] };

    // Maximum quote size is 16K.
    let mut quote: Vec<u8> = Vec::with_capacity(16 * 1024);
    let mut quote_size = 0;

    s_spid.id.copy_from_slice(&spid[..16]);

    sgx_call!("Failed to get quote", result, {
        untrusted::untrusted_get_quote(
            &mut result,
            &report as *const sgx_report_t,
            sgx_quote_sign_type_t::SGX_UNLINKABLE_SIGNATURE,
            &s_spid as *const sgx_spid_t,
            quote.as_mut_ptr() as *mut u8,
            quote.capacity() as u32,
            &mut quote_size,
        )
    });

    unsafe {
        quote.set_len(quote_size as usize);
    }

    Ok(quote)
}

/// Get SPID that can be used to verify the quote later.
pub fn get_spid() -> Result<Vec<u8>> {
    let request = api::services::IasGetSpidRequest::new();
    let mut response: api::services::IasGetSpidResponse =
        bridge::untrusted_call_endpoint(&ClientEndpoint::IASProxyGetSpid, request)?;

    Ok(response.take_spid())
}

/// Verify quote via IAS.
///
/// The quote must have been generated using an SPID returned by `get_spid`.
pub fn verify_quote(quote: Vec<u8>) -> Result<AttestationReport> {
    let mut request = api::services::IasVerifyQuoteRequest::new();
    request.set_quote(quote);

    // Generate random nonce.
    let mut nonce = vec![0u8; 16];
    random::get_random_bytes(&mut nonce)?;
    request.set_nonce(nonce.clone());

    let mut response: api::services::IasVerifyQuoteResponse =
        bridge::untrusted_call_endpoint(&ClientEndpoint::IASProxyVerifyQuote, request)?;

    let mut report = response.take_report();

    let report = AttestationReport::new(
        report.take_body(),
        report.take_signature(),
        report.take_certificates(),
    );

    Ok(report)
}

/// Create attestation report for given public key.
pub fn create_attestation_report_for_public_key(
    quote_context: &QuoteContext,
    nonce: &[u8],
    public_key: &sodalite::BoxPublicKey,
) -> Result<AttestationReport> {
    let quote = get_quote(
        &get_spid()?,
        &quote_context,
        create_report_data_for_public_key(&nonce, &public_key)?,
    )?;

    // Then, contact IAS to get the attestation report.
    Ok(verify_quote(quote)?)
}
