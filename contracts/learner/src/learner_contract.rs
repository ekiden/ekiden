use std::error::Error;

use rusty_machine::learning::lin_reg::LinRegressor;
use rusty_machine::prelude::*;
use serde_cbor;

use libcontract_common::{Address, Contract, ContractError};

use learner_api::LearnerState;

pub struct Learner {
    owner: Address,
    model: LinRegressor,
}

impl Learner {
    pub fn new(owner: Address) -> Learner {
        Learner {
            owner: owner,
            model: LinRegressor::default(),
        }
    }

    pub fn train(
        &mut self,
        inputs: &Matrix<f64>,
        targets: &Vector<f64>,
    ) -> Result<String, ContractError> {
        match self.model.train(inputs, targets) {
            Ok(_) => Ok("The model trained, hooray!".to_string()),
            Err(err) => Err(ContractError::new(err.description())),
        }
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

impl Contract<LearnerState> for Learner {
    fn get_state(&self) -> LearnerState {
        let mut state = LearnerState::new();
        state.set_owner(self.owner.to_string());
        state.set_model(serde_cbor::to_vec(&self.model).expect("Unable to serialize model."));
        state
    }

    /// Create contract instance from serialized state.
    fn from_state(state: &LearnerState) -> Learner {
        Learner {
            owner: Address::from(state.get_owner().to_string()),
            model: serde_cbor::from_slice(state.get_model()).expect("Unable to deserialize model."),
        }
    }
}
