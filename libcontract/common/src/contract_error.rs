use protobuf;

use std::io;
use std::error::Error as StdError;

#[derive(Debug)]
pub struct ContractError {
    pub message: String
}

impl ContractError {
    pub fn new(msg: &str) -> ContractError {
        ContractError {
            message: msg.to_string(),
        }
    }
}

impl From<protobuf::ProtobufError> for ContractError {
    fn from(_e: protobuf::ProtobufError) -> Self {
        ContractError::new("Malformed message")
    }
}

impl From<io::Error> for ContractError {
    fn from(e: io::Error) -> Self {
        ContractError::new(e.description())
    }
}
