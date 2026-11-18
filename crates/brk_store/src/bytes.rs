use std::path::Path;

use brk_error::Result;
use brk_types::Version;
use byteview::ByteView;
use fjall::{Database, FORMAT_VERSION, Keyspace, KeyspaceCreateOptions};

use crate::meta::checked_open;

/// Fixed keys and variable values, committed together by one sorted ingestion.
/// The caller owns the write lifecycle, including its shutdown guard.
#[derive(Clone)]
pub struct ByteStore<const N: usize> {
    keyspace: Keyspace,
    // A keyspace pins its files, but the database owns its background compaction workers.
    _database: Database,
}

impl<const N: usize> ByteStore<N> {
    pub fn import(db: &Database, path: &Path, name: &str, version: Version) -> Result<Self> {
        let keyspace = checked_open(
            &path.join(format!("meta/{name}")),
            Version::new(u32::from(FORMAT_VERSION)) + version,
            || Ok(db.keyspace(name, || KeyspaceCreateOptions::default()
                .compaction_records::<[u8; N], ByteView>())?),
        )?;
        Ok(Self { keyspace, _database: db.clone() })
    }

    pub fn get(&self, key: &[u8; N]) -> Result<Option<ByteView>> {
        Ok(self.keyspace.get_as(key)?)
    }

    pub fn prefix(&self, prefix: &[u8]) -> impl Iterator<Item = Result<([u8; N], ByteView)>> + use<N> {
        self.keyspace.read_view().prefix_as(prefix).map(|row| Ok(row?))
    }

    /// `records` must be in strictly increasing key order.
    pub fn ingest(&mut self, records: impl IntoIterator<Item = ([u8; N], Vec<u8>)>) -> Result<()> {
        let mut ingest = self.keyspace.start_ingestion_as::<[u8; N], ByteView>()?;
        for (key, value) in records {
            ingest.write(key, value)?;
        }
        ingest.finish()?;
        Ok(())
    }
}
