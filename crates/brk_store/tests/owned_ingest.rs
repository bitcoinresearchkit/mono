use brk_error::Result;
use brk_store::{Kind, Mode, Store, open_database};
use brk_types::{AddrHash, AddrIndexTxIndex, TxIndex, TypeIndex, Unit, Version};

fn key(address: u32, transaction: u32) -> AddrIndexTxIndex {
    AddrIndexTxIndex::from((TypeIndex::new(address), TxIndex::new(transaction)))
}

#[test]
fn owned_ingest_merges_puts_and_tombstones() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path();

    {
        let db = open_database(path)?;
        let mut store = Store::import(
            &db,
            path,
            "owned_ingest",
            Version::ZERO,
            Mode::Any,
            Kind::Vec,
        )?;

        store.insert(key(1, 1), Unit);
        store.insert(key(2, 2), Unit);
        store.take_pending_ingest().unwrap().run()?;

        store.remove(key(1, 1));
        store.remove(key(3, 3));
        store.insert(key(4, 4), Unit);
        store.take_pending_ingest().unwrap().run()?;
        assert!(store.get(&key(1, 1))?.is_none());
        assert!(store.get(&key(2, 2))?.is_some());
        assert!(store.get(&key(3, 3))?.is_none());
        assert!(store.get(&key(4, 4))?.is_some());
    }

    {
        let db = open_database(path)?;
        let store: Store<AddrIndexTxIndex, Unit> = Store::import(
            &db,
            path,
            "owned_ingest",
            Version::ZERO,
            Mode::Any,
            Kind::Vec,
        )?;

        assert!(store.get(&key(1, 1))?.is_none());
        assert!(store.get(&key(2, 2))?.is_some());
        assert!(store.get(&key(3, 3))?.is_none());
        assert!(store.get(&key(4, 4))?.is_some());
    }

    Ok(())
}

#[test]
fn vector_pending_preserves_insert_remove_semantics() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path();
    let db = open_database(path)?;
    let mut store = Store::import(
        &db,
        path,
        "vector_pending",
        Version::ZERO,
        Mode::Any,
        Kind::Vec,
    )?;

    let restored = key(3, 3);
    store.insert(restored, Unit);
    store.take_pending_ingest().unwrap().run()?;

    let cancelled = key(1, 1);
    store.insert(cancelled, Unit);
    store.remove(cancelled);
    assert!(store.get(&cancelled)?.is_none());

    let deleted = key(2, 2);
    store.insert(deleted, Unit);
    store.remove(deleted);
    store.remove(deleted);
    assert!(store.get(&deleted)?.is_none());

    store.remove(restored);
    store.insert(restored, Unit);
    store.remove(restored);
    assert!(store.get(&restored)?.is_some());

    store.take_pending_ingest().unwrap().run()?;
    assert!(store.get(&cancelled)?.is_none());
    assert!(store.get(&deleted)?.is_none());
    assert!(store.get(&restored)?.is_some());

    Ok(())
}

#[test]
fn pending_tombstone_hides_persisted_point_value() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path();
    let db = open_database(path)?;
    let mut store = Store::import(
        &db,
        path,
        "pending_tombstone",
        Version::ZERO,
        Mode::Any,
        Kind::Random,
    )?;
    let key = AddrHash::new(42);

    store.insert(key, TypeIndex::new(1));
    store.take_pending_ingest().unwrap().run()?;
    assert!(store.get(&key)?.is_some());

    store.remove(key);
    assert!(store.get(&key)?.is_none());
    store.take_pending_ingest().unwrap().run()?;
    assert!(store.get(&key)?.is_none());

    Ok(())
}
