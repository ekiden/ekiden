use std::io;
use std::fmt;
use std::error::Error as StdError;

use protobuf;

use libcontract_common::ContractError;

#[derive(Debug)]
pub struct Error {
    pub message: String,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Error {
            message: message.to_string()
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        &self.message
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Self {
        Error::new(e.description())
    }
}

impl From<ContractError> for Error {
    fn from(e: ContractError) -> Self {
        Error::new(&e.message)
    }
}

impl From<protobuf::ProtobufError> for Error {
    fn from(_e: protobuf::ProtobufError) -> Self {
        Error::new("Malformed message")
    }
}
