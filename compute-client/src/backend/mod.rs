mod base;

#[cfg(not(feature = "sgx"))]
pub mod web3;

// Re-export.
pub use self::base::ContractClientBackend;

#[cfg(not(feature = "sgx"))]
pub use self::web3::Web3ContractClientBackend;
