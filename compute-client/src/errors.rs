use std::fmt;
use std::error::Error as StdError;

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
