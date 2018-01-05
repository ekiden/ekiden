
#[derive(Debug)]
pub struct ContractError {
    pub message: String
}

impl ContractError {
    pub fn new(msg: String) -> ContractError {
        ContractError {
            message: msg,
        }
    }
}
