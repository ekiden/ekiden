use std::io::{Read, Write};

use protobuf::{self, Message, MessageStatic};

use super::error::Result;

/// A serializer for a specific data type.
pub trait Serializable {
    /// Serialize message of a given type into raw bytes.
    fn write(&self) -> Result<Vec<u8>>;

    /// Write the contents of self into given writer.
    ///
    /// Returns the number of bytes written.
    fn write_to(&self, writer: &mut Write) -> Result<usize>;

    /// Deserialize message of a given type from raw bytes.
    fn read(value: &Vec<u8>) -> Result<Self>
    where
        Self: Sized;

    /// Deserialize message of a given type from reader.
    fn read_from(reader: &mut Read) -> Result<Self>
    where
        Self: Sized;
}

/// Protocol Buffers serializer.
impl<M: Message + MessageStatic> Serializable for M {
    /// Serialize message of a given type into raw bytes.
    fn write(&self) -> Result<Vec<u8>> {
        Ok(self.write_to_bytes()?)
    }

    /// Write the contents of self into given writer.
    ///
    /// Returns the number of bytes written.
    fn write_to(&self, writer: &mut Write) -> Result<usize> {
        self.write_to_writer(writer)?;

        Ok(self.compute_size() as usize)
    }

    /// Deserialize message of a given type from raw bytes.
    fn read(value: &Vec<u8>) -> Result<M> {
        Ok(protobuf::parse_from_bytes(&value)?)
    }

    /// Deserialize message of a given type from reader.
    fn read_from(reader: &mut Read) -> Result<Self> {
        Ok(protobuf::parse_from_reader(reader)?)
    }
}
