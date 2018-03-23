//! RPC client backend.
mod base;

#[cfg(not(target_env = "sgx"))]
pub mod web3;

// Re-export.
pub use self::base::ContractClientBackend;

#[cfg(not(target_env = "sgx"))]
pub use self::web3::Web3ContractClientBackend;
