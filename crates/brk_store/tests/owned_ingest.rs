use brk_store::{Kind, Mode, Store, open_database};
use brk_types::{AddrIndexTxIndex, TxIndex, TypeIndex, Unit, Version};

fn key(address: u32, transaction: u32) -> AddrIndexTxIndex {
    AddrIndexTxIndex::from((TypeIndex::new(address), TxIndex::new(transaction)))
}

#[test]
fn owned_ingest_merges_puts_and_tombstones() -> brk_error::Result<()> {
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
