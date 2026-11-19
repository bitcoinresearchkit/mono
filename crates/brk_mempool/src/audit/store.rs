use std::path::Path;
use bitcoin::hashes::Hash;

use brk_error::{Error, Result};
use brk_exit::Exit;
use brk_store::{ByteStore, open_database};
use brk_types::{BlockHash, Version};

use super::{Audit, Difference, Summary};

/// The daemon's sole audit writer, outside chain checkpoint and rollback ownership.
pub struct AuditStore {
    records: ByteStore<33>,
    exit: Exit,
}

#[derive(Clone)]
pub struct AuditReader(ByteStore<33>);

fn key(kind: u8, hash: BlockHash) -> [u8; 33] {
    let mut key = [kind; 33];
    key[1..].copy_from_slice(&*hash);
    key
}

impl AuditStore {
    pub fn open(path: &Path, exit: &Exit) -> Result<Self> {
        exit.park_if_shutting_down();
        let _guard = exit.lock();
        let db = open_database(path)?;
        Ok(Self { records: ByteStore::import(&db, path, "audits", Version::ONE)?, exit: exit.clone() })
    }

    pub fn reader(&self) -> AuditReader { AuditReader(self.records.clone()) }

    /// First committed observation wins. A retry after an ambiguous I/O result checks both keys.
    pub fn commit(&mut self, audit: &Audit) -> Result<bool> {
        self.exit.park_if_shutting_down();
        let _guard = self.exit.lock();
        let summary_key = key(0, audit.summary.hash);
        let diff_key = key(1, audit.summary.hash);
        match (self.records.get(&summary_key)?, self.records.get(&diff_key)?) {
            (Some(_), Some(_)) => return Ok(false),
            (None, None) => {},
            _ => return Err(Error::Internal("Incomplete committed audit pair")),
        }
        self.records.ingest([
            (summary_key, audit.summary.encode()),
            (diff_key, audit.difference.encode()),
        ])?;
        Ok(true)
    }
}

impl AuditReader {
    pub fn summary(&self, hash: BlockHash) -> Result<Option<Summary>> {
        self.0.get(&key(0, hash))?.map(|value| Summary::decode(hash, &value)).transpose()
    }

    pub fn difference(&self, hash: BlockHash) -> Result<Option<Difference>> {
        self.0.get(&key(1, hash))?.map(|value| Difference::decode(&value)).transpose()
    }

    /// Startup reads just summaries; permanent hash reads do not depend on this scan.
    pub fn summaries(&self) -> impl Iterator<Item = Result<Summary>> + use<> {
        self.0.prefix(&[0]).map(|row| {
            let (key, value) = row?;
            Summary::decode(bitcoin::BlockHash::from_byte_array(key[1..].try_into().unwrap()).into(), &value)
        })
    }
}
