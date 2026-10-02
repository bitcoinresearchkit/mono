use brk_error::{Error, Result};
use brk_store::{PendingIngest, open_database};
use brk_types::Height;
use tempfile::tempdir;

use super::{DeferredStoresCommit, StoresCheckpoint};

#[test]
fn failed_deferred_commit_invalidates_the_previous_checkpoint() -> Result<()> {
    let dir = tempdir()?;
    let checkpoint = StoresCheckpoint::new(dir.path());

    checkpoint
        .begin(Height::new(41))?
        .persist(|| Ok(()))?
        .publish()?;
    assert_eq!(checkpoint.next_height()?, Some(Height::new(42)));

    let pending = checkpoint.begin(Height::new(42))?;
    assert_eq!(checkpoint.next_height()?, None);
    let db = open_database(dir.path())?;
    let ingests = vec![PendingIngest::new(|| {
        Err(Error::Internal("simulated ingest failure"))
    })];

    assert!(
        DeferredStoresCommit::new(db, ingests, pending)
            .persist()
            .is_err()
    );
    assert_eq!(StoresCheckpoint::new(dir.path()).next_height()?, None);
    Ok(())
}
