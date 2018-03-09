#![feature(core_intrinsics)]
#![feature(use_extern_macros)]

extern crate bsdiff;
extern crate bzip2;
#[macro_use]
extern crate lazy_static;
extern crate protobuf;
extern crate sodalite;

extern crate ekiden_common;
extern crate ekiden_enclave_trusted;
extern crate key_manager_client;

mod generated;

mod crypto;
mod diffs;
pub mod ecalls;

pub mod handle;
pub use handle::DatabaseHandle;

/// Database interface exposed to contracts.
pub trait Database {
    /// Returns true if the database contains a value for the specified key.
    fn contains_key(&self, key: &[u8]) -> bool;

    /// Fetch entry with given key.
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;

    /// Update entry with given key.
    ///
    /// If the database did not have this key present, [`None`] is returned.
    ///
    /// If the database did have this key present, the value is updated, and the old value is
    /// returned.
    fn insert(&mut self, key: &[u8], value: &[u8]) -> Option<Vec<u8>>;

    /// Remove entry with given key, returning the value at the key if the key was previously
    /// in the database.
    fn remove(&mut self, key: &[u8]) -> Option<Vec<u8>>;
}
