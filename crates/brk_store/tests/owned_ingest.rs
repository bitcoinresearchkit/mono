use brk_error::Result;
use brk_store::{Kind, Store, open_database};
use brk_types::{
    AddrIndexOutPoint, AddrIndexTxIndex, OutPoint, TxIndex, TypeIndex, Unit, Version, Vout,
};
use tempfile::tempdir;

fn key(address: u32, transaction: u32) -> AddrIndexTxIndex {
    AddrIndexTxIndex::from((TypeIndex::new(address), TxIndex::new(transaction)))
}

#[test]
fn owned_ingest_preserves_pending_cancellation_tombstones_and_reopened_values() -> Result<()> {
    let dir = tempdir()?;
    let path = dir.path();

    {
        let db = open_database(path)?;
        let mut store = Store::import(&db, path, "owned_ingest", Version::ZERO, Kind::Vec)?;

        store.insert(key(1, 1), Unit);
        store.insert(key(2, 2), Unit);
        store.take_pending_ingest().unwrap().run()?;

        store.remove(key(1, 1));
        store.remove(key(3, 3));
        store.insert(key(4, 4), Unit);

        let cancelled = key(5, 5);
        store.insert(cancelled, Unit);
        store.remove(cancelled);
        assert!(store.get(&cancelled)?.is_none());

        let deleted = key(6, 6);
        store.insert(deleted, Unit);
        store.remove(deleted);
        store.remove(deleted);
        assert!(store.get(&deleted)?.is_none());

        let restored = key(2, 2);
        store.remove(restored);
        store.insert(restored, Unit);
        store.remove(restored);
        assert!(store.get(&restored)?.is_some());

        store.take_pending_ingest().unwrap().run()?;
        assert!(store.get(&key(1, 1))?.is_none());
        assert!(store.get(&key(2, 2))?.is_some());
        assert!(store.get(&key(3, 3))?.is_none());
        assert!(store.get(&key(4, 4))?.is_some());
        assert!(store.get(&cancelled)?.is_none());
        assert!(store.get(&deleted)?.is_none());
    }

    {
        let db = open_database(path)?;
        let store: Store<AddrIndexTxIndex, Unit> =
            Store::import(&db, path, "owned_ingest", Version::ZERO, Kind::Vec)?;

        assert!(store.get(&key(1, 1))?.is_none());
        assert!(store.get(&key(2, 2))?.is_some());
        assert!(store.get(&key(3, 3))?.is_none());
        assert!(store.get(&key(4, 4))?.is_some());
        assert!(store.get(&key(5, 5))?.is_none());
        assert!(store.get(&key(6, 6))?.is_none());
    }

    Ok(())
}

#[test]
fn key_only_prefix_preserves_order_after_reopen() -> Result<()> {
    let dir = tempdir()?;
    let key = |address, transaction| {
        AddrIndexOutPoint::from((
            TypeIndex::new(address),
            OutPoint::new(TxIndex::new(transaction), Vout::from(2_u16)),
        ))
    };
    {
        let db = open_database(dir.path())?;
        let mut store = Store::import(&db, dir.path(), "outputs", Version::ZERO, Kind::Vec)?;
        for transaction in [5, 1, 3] {
            store.insert(key(42, transaction), Unit);
        }
        store.insert(key(43, 2), Unit);
        store.take_pending_ingest().unwrap().run()?;
        store.remove(key(42, 3));
        store.take_pending_ingest().unwrap().run()?;
    }
    let db = open_database(dir.path())?;
    let store: Store<AddrIndexOutPoint, Unit> =
        Store::import(&db, dir.path(), "outputs", Version::ZERO, Kind::Vec)?;
    let expected = vec![key(42, 1), key(42, 5)];
    assert_eq!(
        store
            .prefix(TypeIndex::new(42))
            .map(|(k, _)| k)
            .collect::<Vec<_>>(),
        expected
    );
    assert_eq!(
        store
            .prefix(TypeIndex::new(42))
            .rev()
            .map(|(k, _)| k)
            .collect::<Vec<_>>(),
        expected.into_iter().rev().collect::<Vec<_>>()
    );
    assert!(store.get(&key(42, 1))?.is_some());
    assert!(store.get(&key(42, 3))?.is_none());
    Ok(())
}
