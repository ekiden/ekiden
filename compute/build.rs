extern crate ekiden_core_edl;
extern crate ekiden_tools;

fn main() {
    ekiden_tools::build_untrusted(ekiden_core_edl::edl());
}
