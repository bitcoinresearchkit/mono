use std::{fs, path::Path};

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
    /// Opens the store at `path`, first merging the store copies in `path/import/`, in name
    /// order, each one's records replacing those already there, then removing that directory.
    /// Copies come from other servers (fleet's swap): a crash before the removal merges them again,
    /// to the same result.
    pub fn open(path: &Path, exit: &Exit) -> Result<Self> {
        exit.park_if_shutting_down();
        let _guard = exit.lock();
        let mut records = open_records(path)?;
        let imports = path.join("import");
        if imports.try_exists()? {
            let mut copies = fs::read_dir(&imports)?.map(|entry| Ok(entry?.path())).collect::<Result<Vec<_>>>()?;
            copies.sort();
            for copy in copies {
                records.ingest(check_copy(&copy)?)?;
            }
            fs::remove_dir_all(&imports)?;
        }
        Ok(Self { records, exit: exit.clone() })
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

fn open_records(path: &Path) -> Result<ByteStore<33>> {
    ByteStore::import(&open_database(path)?, path, "audits", Version::ONE)
}

/// Every record of the audit store at `path`, in key order, each one decoded and every summary
/// paired with its difference: what a copy must pass before it is merged.
pub fn check_copy(path: &Path) -> Result<Vec<([u8; 33], Vec<u8>)>> {
    let check = || {
        // Opening would create an empty store: a copy that isn't there must fail instead.
        if !path.join("fjall").try_exists()? {
            return Err(Error::NotFound("no audit store here".into()));
        }
        let rows = open_records(path)?.prefix(&[]).map(|row| row.map(|(key, value)| (key, value.to_vec())))
            .collect::<Result<Vec<_>>>()?;
        let (summaries, differences) = rows.split_at(rows.partition_point(|(key, _)| key[0] == 0));
        if summaries.len() != differences.len() {
            return Err(Error::InvalidData("unpaired audit records"));
        }
        for ((summary_key, summary), (diff_key, difference)) in summaries.iter().zip(differences) {
            if diff_key[0] != 1 || summary_key[1..] != diff_key[1..] {
                return Err(Error::InvalidData("unpaired audit records"));
            }
            Summary::decode(bitcoin::BlockHash::from_byte_array(summary_key[1..].try_into().unwrap()).into(), summary)?;
            Difference::decode(difference)?;
        }
        Ok(rows)
    };
    check().map_err(|error: Error| error.opening_storage(path))
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
