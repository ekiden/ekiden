use protobuf::{self, Message, MessageStatic};

use ekiden_enclave_common::error::Result;

pub trait Serializable {
    fn write(value: &Self) -> Result<Vec<u8>>;

    fn read(value: &Vec<u8>) -> Result<Self>
    where
        Self: Sized;
}

impl<M: Message + MessageStatic> Serializable for M {
    fn write(value: &M) -> Result<Vec<u8>> {
        Ok(value.write_to_bytes()?)
    }

    fn read(value: &Vec<u8>) -> Result<M> {
        let value: M = protobuf::parse_from_bytes(&value)?;
        Ok(value)
    }
}
