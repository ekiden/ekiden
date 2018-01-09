
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
