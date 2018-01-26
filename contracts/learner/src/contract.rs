use std::error::Error;

use protobuf;
use rusty_machine::prelude::*;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_cbor;

use libcontract_common::{Address, Contract, ContractError};

use api::LearnerState;

pub struct Learner<M: SupModel<Matrix<f64>, Vector<f64>> + Serialize + DeserializeOwned> {
    owner: Address,
    model: M,
    inputs: Vec<String>,
    targets: Vec<String>,
}

impl<M: SupModel<Matrix<f64>, Vector<f64>> + Serialize + DeserializeOwned> Learner<M> {
    pub fn new(owner: Address, model: M, inputs: Vec<String>, targets: Vec<String>) -> Learner<M> {
        Learner {
            owner: owner,
            model: model,
            inputs: inputs,
            targets: targets,
        }
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

    pub fn get_inputs(&self) -> Result<&Vec<String>, ContractError> {
        Ok(&self.inputs)
    }

    pub fn get_targets(&self) -> Result<&Vec<String>, ContractError> {
        Ok(&self.targets)
    }
}

impl<M: SupModel<Matrix<f64>, Vector<f64>> + Serialize + DeserializeOwned> Contract<LearnerState>
    for Learner<M>
{
    fn get_state(&self) -> LearnerState {
        let mut state = LearnerState::new();
        state.set_owner(self.owner.to_string());
        state.set_model(serde_cbor::to_vec(&self.model).expect("Unable to serialize model."));
        state.set_inputs(protobuf::RepeatedField::from_vec(self.inputs.clone()));
        state.set_targets(protobuf::RepeatedField::from_vec(self.targets.clone()));
        state
    }

    fn from_state(state: &LearnerState) -> Learner<M> {
        Learner {
            owner: Address::from(state.get_owner().to_string()),
            model: serde_cbor::from_slice(state.get_model()).expect("Unable to deserialize model."),
            inputs: state.get_inputs().to_vec(),
            targets: state.get_targets().to_vec(),
        }
    }
}
