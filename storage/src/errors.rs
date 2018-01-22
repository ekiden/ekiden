use hyper;
use serde_json;
use std::string;

#[derive(Debug)]
pub enum Error {
    HyperError(hyper::Error),
    HyperUriError(hyper::error::UriError),
    JsonError(serde_json::Error),
    StringError(string::FromUtf8Error),
}

impl From<hyper::Error> for Error {
    fn from(error: hyper::Error) -> Self {
        Error::HyperError(error)
    }
}

impl From<hyper::error::UriError> for Error {
    fn from(error: hyper::error::UriError) -> Self {
        Error::HyperUriError(error)
    }
}

impl From<serde_json::Error> for Error {
    fn from(error: serde_json::Error) -> Self {
        Error::JsonError(error)
    }
}

impl From<string::FromUtf8Error> for Error {
    fn from(error: string::FromUtf8Error) -> Self {
        Error::StringError(error)
    }
}
