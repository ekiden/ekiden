use std::error::Error;

use protobuf;
use rusty_machine::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_cbor;

use libcontract_common::{Address, Contract, ContractError};

use api::LearnerState;

pub struct Learner<M> {
    owner: Address,
    model: M,
}

impl<M: SupModel<Matrix<f64>, Vector<f64>> + Serialize + DeserializeOwned> Learner<M> {
    pub fn new(owner: Address, model: M) -> Result<Learner<M>, ContractError> {
        Ok(Learner {
            owner: owner,
            model: model,
        })
    }

    pub fn train(
        &mut self,
        inputs: &Matrix<f64>,
        targets: &Vector<f64>,
    ) -> Result<(), ContractError> {
        self.model
            .train(inputs, targets)
            .map(|_| ())
            .map_err(|err| ContractError::new(err.description()))
    }

    pub fn infer(&self, inputs: &Matrix<f64>) -> Result<Vector<f64>, ContractError> {
        self.model
            .predict(inputs)
            .map_err(|err| ContractError::new(err.description()))
    }

    pub fn get_owner(&self) -> Result<&Address, ContractError> {
        Ok(&self.owner)
    }
}

impl<M: SupModel<Matrix<f64>, Vector<f64>> + Serialize + DeserializeOwned> Contract<LearnerState>
    for Learner<M>
{
    fn get_state(&self) -> LearnerState {
        let mut state = LearnerState::new();
        state.set_owner(self.owner.to_string());
        state.set_model(serde_cbor::to_vec(&self.model).expect("Unable to serialize model."));
        state
    }

    fn from_state(state: &LearnerState) -> Learner<M> {
        Learner {
            owner: Address::from(state.get_owner().to_string()),
            model: serde_cbor::from_slice(state.get_model()).expect("Unable to deserialize model."),
        }
    }
}
