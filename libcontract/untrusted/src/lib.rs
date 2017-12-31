extern crate sgx_types;
extern crate sgx_urts;
extern crate protobuf;

pub mod common;
pub mod enclave;
pub mod errors;
pub mod generated;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
