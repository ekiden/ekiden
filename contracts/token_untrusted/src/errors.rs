use std::fmt;
use std::error::Error as StdError;

use protobuf;

#[derive(Debug)]
pub enum Error {
    ParseError,
    SgxError
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError => f.write_str("ParseError"),
            Error::SgxError => f.write_str("SgxError"),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParseError => "RPC message parse error",
            Error::SgxError => "SGX execution error",
        }
    }
}

impl From<protobuf::ProtobufError> for Error {
    fn from(_e: protobuf::ProtobufError) -> Self {
        Error::ParseError
    }
}
