use super::Result;

/// Trait that should be implemented by all contracts.
pub trait Contract<State> {
    /// Get serializable contract state.
    fn get_state(&self) -> State;

    /// Create contract instance from serialized state.
    fn from_state(state: &State) -> Self;
}

/// Performs contract operations on serialized state.
pub fn with_contract_state<C, State, Handler>(state: &State, handler: Handler) -> Result<State>
where
    C: Contract<State>,
    Handler: Fn(&mut C) -> Result<()>,
{
    let mut contract = C::from_state(&state);
    handler(&mut contract)?;

    Ok(contract.get_state())
}

#[derive(Debug, PartialEq)]
pub struct Address {
    value: String,
}

impl Address {
    pub fn from(addr: String) -> Address {
        Address { value: addr }
    }

    pub fn as_str(&self) -> &str {
        self.value.as_str()
    }
}

impl ToString for Address {
    fn to_string(&self) -> String {
        self.value.clone()
    }
}
