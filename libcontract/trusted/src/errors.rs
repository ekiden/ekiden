use std::fmt;
use std::error::Error as StdError;

use protobuf;

#[derive(Debug)]
pub enum InternalError {
    ParseError,
}

impl fmt::Display for InternalError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            InternalError::ParseError => f.write_str("ParseError"),
        }
    }
}

impl StdError for InternalError {
    fn description(&self) -> &str {
        match *self {
            InternalError::ParseError => "RPC message parse error",
        }
    }
}

impl From<protobuf::ProtobufError> for InternalError {
    fn from(_e: protobuf::ProtobufError) -> Self {
        InternalError::ParseError
    }
}
