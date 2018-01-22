use sgx_types::*;

/// OCALLs defined by the Ekiden enclave specification.
extern "C" {
    /// Proxy for sgx_init_quote.
    pub fn untrusted_init_quote(
        result: *mut sgx_status_t,
        p_target_info: *mut sgx_target_info_t,
        p_gid: *mut sgx_epid_group_id_t,
    ) -> sgx_status_t;

    /// Proxy for sgx_get_quote.
    pub fn untrusted_get_quote(
        result: *mut sgx_status_t,
        p_report: *const sgx_report_t,
        quote_type: sgx_quote_sign_type_t,
        p_spid: *const sgx_spid_t,
        p_nonce: *const sgx_quote_nonce_t,
        p_qe_report: *mut sgx_report_t,
        p_quote: *mut u8,
        quote_capacity: u32,
        quote_size: *mut u32,
    ) -> sgx_status_t;
}
