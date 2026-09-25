use std::fs;

use fjall::{Database, Error, KeyspaceCreateOptions, Result};
use tempfile::tempdir;

#[test]
fn db_lock() -> Result<()> {
    let folder = tempdir()?;

    let db = Database::builder(&folder).open()?;
    let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;

    let mut ingestion = tree.start_ingestion()?;
    ingestion.write("asd", "def")?;
    ingestion.finish()?;

    drop(db);

    assert!(matches!(
        Database::builder(&folder).open(),
        Err(Error::Locked),
    ));

    Ok(())
}

#[test]
fn lock_error_wins_over_an_invalid_marker() -> Result<()> {
    let folder = tempdir()?;
    let database = Database::builder(&folder).open()?;
    fs::write(folder.path().join("version"), b"FJL\x08")?;

    assert!(matches!(
        Database::builder(&folder).open(),
        Err(Error::Locked),
    ));

    drop(database);
    assert!(matches!(
        Database::builder(&folder).open(),
        Err(Error::InvalidVersion),
    ));

    Ok(())
}
