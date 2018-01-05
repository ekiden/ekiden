use std::fmt;
use std::error::Error as StdError;

use protobuf;

use libcontract_common::api::Response_Code;

#[derive(Debug)]
pub enum Error {
    ParseError,
    ResponseError(Response_Code, String),
    SgxError
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError => f.write_str("ParseError"),
            Error::ResponseError(code, ref message) => f.write_str(format!("ResponseError({:?}, {})", code, message).as_str()),
            Error::SgxError => f.write_str("SgxError"),
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParseError => "RPC message parse error",
            Error::ResponseError(_, _) => "RPC call returned an error",
            Error::SgxError => "SGX execution error",
        }
    }
}

impl From<protobuf::ProtobufError> for Error {
    fn from(_e: protobuf::ProtobufError) -> Self {
        Error::ParseError
    }
}
