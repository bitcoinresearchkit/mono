use fjall::Database;

#[test]
fn values_and_tombstones_survive_recovery() -> fjall::Result<()> {
    let directory = tempfile::tempdir()?;

    {
        let database = Database::builder(&directory).open()?;
        let keyspace = database.keyspace("default", Default::default)?;

        let mut ingestion = keyspace.start_ingestion()?;
        ingestion.write(b"a", b"first")?;
        ingestion.write(b"b", b"second")?;
        ingestion.finish()?;

        let mut ingestion = keyspace.start_ingestion()?;
        ingestion.write_weak_tombstone(b"a")?;
        ingestion.write(b"c", b"third")?;
        ingestion.finish()?;

        assert!(keyspace.get(b"a")?.is_none());
        assert_eq!(keyspace.get(b"b")?.as_deref(), Some(b"second".as_slice()));
        assert_eq!(keyspace.get(b"c")?.as_deref(), Some(b"third".as_slice()));
    }

    {
        let database = Database::builder(&directory).open()?;
        let keyspace = database.keyspace("default", Default::default)?;

        assert!(keyspace.get(b"a")?.is_none());
        assert_eq!(keyspace.get(b"b")?.as_deref(), Some(b"second".as_slice()));
        assert_eq!(keyspace.get(b"c")?.as_deref(), Some(b"third".as_slice()));
    }

    Ok(())
}

#[test]
fn independent_keyspaces_ingest_concurrently() -> fjall::Result<()> {
    let directory = tempfile::tempdir()?;

    {
        let database = Database::builder(&directory).open()?;
        let first = database.keyspace("first", Default::default)?;
        let second = database.keyspace("second", Default::default)?;

        let (first_result, second_result) = std::thread::scope(|scope| {
            let first = scope.spawn(|| {
                let mut ingestion = first.start_ingestion()?;
                ingestion.write(b"a", b"first")?;
                ingestion.finish()
            });
            let second = scope.spawn(|| {
                let mut ingestion = second.start_ingestion()?;
                ingestion.write(b"b", b"second")?;
                ingestion.finish()
            });

            (first.join().unwrap(), second.join().unwrap())
        });
        first_result?;
        second_result?;
    }

    let database = Database::builder(&directory).open()?;
    let first = database.keyspace("first", Default::default)?;
    let second = database.keyspace("second", Default::default)?;
    assert_eq!(first.get(b"a")?.as_deref(), Some(b"first".as_slice()));
    assert_eq!(second.get(b"b")?.as_deref(), Some(b"second".as_slice()));

    Ok(())
}
