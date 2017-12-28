extern crate protobuf;

pub mod address;
pub mod contract;
pub mod contract_error;
pub mod generated;

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
