contract_api! {
    metadata {
        name = learner;
        version = "0.1.0";
        state_type = LearnerState;
    }

    rpc create(CreateRequest) -> (state, CreateResponse);

    rpc train(state, TrainingRequest) -> (state, TrainingResponse);

    rpc infer(state, InferenceRequest) -> InferenceResponse;
}
