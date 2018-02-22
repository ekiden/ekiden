use protobuf::{self, Message, MessageStatic};

use super::error::Result;

/// A serializer for a specific data type.
pub trait Serializable {
    /// Serialize message of a given type into raw bytes.
    fn write(value: &Self) -> Result<Vec<u8>>;

    /// Deserialize message of a given type from raw bytes.
    fn read(value: &Vec<u8>) -> Result<Self>
    where
        Self: Sized;
}

/// Protocol Buffers serializer.
impl<M: Message + MessageStatic> Serializable for M {
    /// Serialize message of a given type into raw bytes.
    fn write(value: &M) -> Result<Vec<u8>> {
        Ok(value.write_to_bytes()?)
    }

    /// Deserialize message of a given type from raw bytes.
    fn read(value: &Vec<u8>) -> Result<M> {
        let value: M = protobuf::parse_from_bytes(&value)?;
        Ok(value)
    }
}
