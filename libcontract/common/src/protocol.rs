/// Secure channel initialization request.
pub const METHOD_CHANNEL_INIT: &'static str = "_channel_init";
/// Secure channel client to contract attestataion request.
pub const METHOD_CHANNEL_ATTEST_CLIENT: &'static str = "_channel_attest_client";
/// Secure channel teardown request.
pub const METHOD_CHANNEL_CLOSE: &'static str = "_channel_close";
/// Diff two states.
pub const METHOD_STATE_DIFF: &'static str = "_state_diff";
/// Apply a diff to a state.
pub const METHOD_STATE_APPLY: &'static str = "_state_apply";
