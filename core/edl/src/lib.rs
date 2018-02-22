#[macro_use]
extern crate ekiden_tools;

extern crate ekiden_db_edl;
extern crate ekiden_rpc_edl;

define_edl! {
    use ekiden_rpc_edl;
    use ekiden_db_edl;

    "core.edl"
}
