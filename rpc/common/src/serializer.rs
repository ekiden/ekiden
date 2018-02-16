use protobuf::{self, Message, MessageStatic};

use ekiden_enclave_common::error::Result;

/// A serializer for a specific data type.
pub trait Serializer<Message> {
    /// Serialize message of a given type into raw bytes.
    fn write(&self, message: &Message) -> Result<Vec<u8>>;

    /// Deserialize message of a given type from raw bytes.
    fn read(&self, bytes: &Vec<u8>) -> Result<Message>;
}

/// Protocol Buffers serializer.
pub struct ProtocolBuffersSerializer;

impl<M: Message + MessageStatic> Serializer<M> for ProtocolBuffersSerializer {
    /// Serialize message of a given type into raw bytes.
    fn write(&self, message: &M) -> Result<Vec<u8>> {
        Ok(message.write_to_bytes()?)
    }

    /// Deserialize message of a given type from raw bytes.
    fn read(&self, bytes: &Vec<u8>) -> Result<M> {
        let message: M = protobuf::parse_from_bytes(&bytes)?;
        Ok(message)
    }
}
