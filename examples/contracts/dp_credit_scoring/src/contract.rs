use std::error::Error;

use dpml;
use ndarray::{Array1, Array2};
use serde_cbor;
use std;

use libcontract_common::{Address, Contract, ContractError};

use api::LearnerState;

pub struct Learner {
    owner: Address,
    model: Model,
}

#[derive(Serialize, Deserialize)]
pub struct Model {
    params: Array2<f64>,
}

impl Learner {
    pub fn new(owner: Address) -> Learner {
        Learner {
            owner: owner,
            model: Model {
                params: Array2::default((4, 2)),
            },
        }
    }

    pub fn train(&mut self, x: &Array2<f64>, y: &Array2<f64>) -> Result<(), ContractError> {
        let params = dpml::dp_logistic_regression(
            x,
            y,
            0.0001, // weight decay
            0.01,   // learning rate
            1.0,    // eps
            1.0,    // delta
        );
        self.model.params = params;
        Ok(())
    }

    pub fn infer(&self, inputs: &Array2<f64>) -> Result<Array2<f64>, ContractError> {
        Ok(inputs
            .dot(&self.model.params)
            .map(|&v| 1.0 / (1.0 + std::f64::consts::E.powf(-v))))
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

    fn from_state(state: &LearnerState) -> Learner {
        Learner {
            owner: Address::from(state.get_owner().to_string()),
            model: serde_cbor::from_slice(state.get_model()).expect("Unable to deserialize model."),
        }
    }
}
