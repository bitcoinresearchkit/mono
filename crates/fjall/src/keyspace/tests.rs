use lsm_tree::Error as TreeError;
use tempfile::tempdir;

use crate::{Database, Error, KeyspaceCreateOptions, Result};

#[test]
fn compaction_uses_configured_record_widths() -> Result<()> {
    let directory = tempdir()?;
    let database = Database::builder(&directory).open()?;
    let keyspace = database.keyspace("records", || {
        KeyspaceCreateOptions::default().compaction_records::<[u8; 8], [u8; 4]>()
    })?;

    // Prepare runs directly so the background worker cannot race this check.
    for generation in 0..4_u8 {
        let mut ingest = keyspace.inner.tree.ingestion()?;
        ingest.write(1_u64.to_be_bytes(), [generation; 3])?;
        ingest.finish()?;
    }
    let version = keyspace.inner.tree.current_version_id();
    assert!(matches!(
        keyspace.compact(),
        Err(Error::Storage(TreeError::InvalidRecordLength {
            expected: 4,
            actual: 3,
        }))
    ));
    assert_eq!(keyspace.inner.tree.current_version_id(), version);
    assert_eq!(
        keyspace.get_as::<[u8; 3]>(&1_u64.to_be_bytes())?,
        Some([3; 3])
    );
    Ok(())
}
