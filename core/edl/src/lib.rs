#[macro_use]
extern crate ekiden_tools;

extern crate ekiden_db_edl;
extern crate ekiden_rpc_edl;

define_edl! {
    use ekiden_rpc_edl;
    use ekiden_db_edl;

    "core.edl",

    // Define core EDLs required by rust-sgx-sdk. These are copied over from there so
    // that we don't need to bring in the whole SDK just to get these EDLs.
    //
    // It would be much better if EDLs were instead provided in crates, similar to what
    // Ekiden does as this would mean we could easily import them.
    "sgx_tstd.edl",
    "sgx_stdio.edl",
    "sgx_backtrace.edl",
    "sgx_time.edl",
}
