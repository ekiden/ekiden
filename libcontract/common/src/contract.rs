use protobuf::Message;

use super::contract_error::ContractError;

/// Trait that should be implemented by all contracts.
pub trait Contract<State: Message> {
    /// Get serializable contract state.
    fn get_state(&self) -> State;

    /// Create contract instance from serialized state.
    fn from_state(state: &State) -> Self;
}

/// Performs contract operations on serialized state.
pub fn with_contract_state<C, State, Handler>(state: &State, handler: Handler) -> Result<State, ContractError>
    where C: Contract<State>,
          State: Message,
          Handler: Fn(&mut C) -> Result<(), ContractError> {

    let mut contract = C::from_state(&state);
    handler(&mut contract)?;

    Ok(contract.get_state())
}
