use fjall::{Database, KeyspaceCreateOptions};

#[test]
fn db_lock() -> fjall::Result<()> {
    let folder = tempfile::tempdir()?;

    let db = Database::builder(&folder).open()?;
    let tree = db.keyspace("default", KeyspaceCreateOptions::default)?;

    let mut ingestion = tree.start_ingestion()?;
    ingestion.write("asd", "def")?;
    ingestion.finish()?;

    drop(db);

    assert!(matches!(
        Database::builder(&folder).open(),
        Err(fjall::Error::Locked),
    ));

    Ok(())
}

#[test]
fn lock_error_wins_over_an_invalid_marker() -> fjall::Result<()> {
    let folder = tempfile::tempdir()?;
    let database = Database::builder(&folder).open()?;
    std::fs::write(folder.path().join("version"), b"invalid")?;

    assert!(matches!(
        Database::builder(&folder).open(),
        Err(fjall::Error::Locked),
    ));

    drop(database);
    assert!(matches!(
        Database::builder(&folder).open(),
        Err(fjall::Error::InvalidVersion),
    ));

    Ok(())
}
