use crate::{Kind, Store, open_database};
use brk_error::Result;
use brk_types::{AddrIndexTxIndex, TxIndex, TypeIndex, Unit, Version};
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
