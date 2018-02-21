use std::sync::{SgxMutex, SgxMutexGuard};

use protobuf::{self, Message};

use ekiden_common::error::Result;

use super::crypto;
use super::serializer::Serializable;

/// Database interface.
// TODO: Make it easy to retrieve diffs (e.g. `export` should return oplog records).
pub struct Db {
    /// Current database state.
    // TODO: Make it a proper key-value store.
    state: Vec<u8>,
    // TODO: Track dirty status of individual keys (possibly using an oplog?).
    dirty: bool,
}

lazy_static! {
    // Global database object.
    static ref DB: SgxMutex<Db> = SgxMutex::new(Db::new());
}

impl Db {
    /// Construct new database interface.
    fn new() -> Self {
        Db {
            state: vec![],
            dirty: false,
        }
    }

    /// Get global database interfaceinstance.
    ///
    /// Calling this method will take a lock on the global instance, which will
    /// be released once the value goes out of scope.
    pub fn instance<'a>() -> SgxMutexGuard<'a, Db> {
        DB.lock().unwrap()
    }

    /// Fetch entry with given key.
    pub fn get<V>(&self, key: &str) -> Result<V>
    where
        V: Serializable,
    {
        V::read(self.get_raw(&key))
    }

    /// Update entry with given key.
    pub fn set<V>(&mut self, key: &str, value: V) -> Result<()>
    where
        V: Serializable,
    {
        Ok(self.set_raw(&key, V::write(&value)?))
    }

    /// Fetch entry with given key.
    pub fn get_raw(&self, _key: &str) -> &Vec<u8> {
        // TODO: Key is currently ignored. Make this a proper key-value store.
        &self.state
    }

    /// Update entry with given key.
    pub fn set_raw(&mut self, _key: &str, value: Vec<u8>) {
        // TODO: Key is currently ignored. Make this a proper key-value store.
        self.state = value;
        self.dirty = true;
    }

    /// Import database.
    pub(crate) fn import(&mut self, state: Vec<u8>) -> Result<()> {
        self.state = crypto::decrypt_state(&protobuf::parse_from_bytes(&state)?)?;
        self.dirty = false;

        Ok(())
    }

    /// Export database.
    ///
    /// If nothing was modified since the last import, this method will return an empty
    /// vector.
    pub(crate) fn export(&self) -> Result<Vec<u8>> {
        if !self.dirty {
            // Database has not changed, we don't need to export anything.
            Ok(vec![])
        } else {
            Ok(crypto::encrypt_state(self.state.clone())?.write_to_bytes()?)
        }
    }
}
