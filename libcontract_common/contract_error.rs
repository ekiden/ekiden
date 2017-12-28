use std::string::String;

#[derive(Debug)]
pub struct ContractError {
  message: String
}

impl ContractError {
  pub fn new(msg: String) -> ContractError {
    ContractError {
      message: msg,
    }
  }
}
