use std::error::Error as StdError;
use std::fmt;

use protobuf;

use libcontract_common::api::PlainClientResponse_Code;

#[derive(Debug)]
pub enum Error {
    ParseError,
    RpcRouterInvalidEndpoint,
    RpcRouterCallFailed,
    ResponseError(PlainClientResponse_Code, String),
    SgxError,
    OtherError(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::ParseError => f.write_str("ParseError"),
            Error::ResponseError(code, ref message) => {
                f.write_str(format!("ResponseError({:?}, {})", code, message).as_str())
            }
            Error::RpcRouterInvalidEndpoint => f.write_str("RpcRouterInvalidEndpoint"),
            Error::RpcRouterCallFailed => f.write_str("RpcRouterCallFailed"),
            Error::SgxError => f.write_str("SgxError"),
            Error::OtherError(ref message) => {
                f.write_str(format!("OtherError({})", message).as_str())
            }
        }
    }
}

impl StdError for Error {
    fn description(&self) -> &str {
        match *self {
            Error::ParseError => "RPC message parse error",
            Error::RpcRouterInvalidEndpoint => "RPC router: invalid endpoint",
            Error::RpcRouterCallFailed => "RPC router: call failed",
            Error::ResponseError(_, _) => "RPC call returned an error",
            Error::SgxError => "SGX execution error",
            Error::OtherError(_) => "Other error",
        }
    }
}

impl From<protobuf::ProtobufError> for Error {
    fn from(_e: protobuf::ProtobufError) -> Self {
        Error::ParseError
    }
}
