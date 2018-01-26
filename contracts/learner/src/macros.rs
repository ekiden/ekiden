#[macro_export]
macro_rules! check_owner {
    ($model:ty, $state:ident, $req:ident) => {
        {
            let learner = Learner::<$model>::from_state(&$state);
            if !Address::from($req.get_requester().to_string()).eq(learner.get_owner()?) {
                return Err(ContractError::new("Insufficient permissions."));
            }
            learner
        }
    }
}

#[macro_export]
macro_rules! unpack_target_vec {
    ($examples:ident, $targets:expr) => {
        unpack_feature_vector(
            $examples,
            $targets.first().ok_or(ContractError::new("No targets specified."))?
        )
    }
}
