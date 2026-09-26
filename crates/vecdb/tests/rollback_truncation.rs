//! Rollback of stamps that included a `truncate_if_needed_at` shrink.
//! (The `rollback_with_truncation` test in `tests/rollback.rs` is misnamed —
//! it never actually calls `truncate_if_needed_at`.)

use std::fs;

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{ImportOptions, Result, Stamp, StoredVec, Version};

fn setup_db() -> Result<(Database, TempDir)> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    Ok((db, temp))
}

fn import_with_changes<V>(db: &Database, name: &str, changes: u16) -> Result<V>
where
    V: StoredVec<I = usize, T = u32>,
{
    let mut options: ImportOptions = (db, name, Version::TWO).into();
    options = options.with_saved_stamped_changes(changes);
    V::forced_import_with(options)
}

fn run_rollback_after_pure_truncate<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = import_with_changes::<V>(&db, "pure_truncate", 10)?;

    for i in 0..8 {
        vec.push(i);
    }
    vec.stamped_write_with_changes(Stamp::new(1))?;
    assert_eq!(vec.len(), 8);
    assert_eq!(vec.collect(), (0..8).collect::<Vec<u32>>());

    vec.truncate_if_needed_at(5)?;
    vec.stamped_write_with_changes(Stamp::new(2))?;
    assert_eq!(vec.len(), 5);
    assert_eq!(vec.collect(), (0..5).collect::<Vec<u32>>());

    vec.rollback()?;
    assert_eq!(vec.len(), 8, "len mismatch after rollback of truncation");
    assert_eq!(
        vec.collect(),
        (0..8).collect::<Vec<u32>>(),
        "data mismatch after rollback of truncation"
    );
    assert_eq!(vec.stamp(), Stamp::new(1));

    Ok(())
}

fn run_rollback_after_truncate_and_push<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = import_with_changes::<V>(&db, "truncate_push", 10)?;

    for i in 0..5 {
        vec.push(i);
    }
    vec.stamped_write_with_changes(Stamp::new(1))?;

    vec.truncate_if_needed_at(3)?;
    vec.push(100);
    vec.push(200);
    vec.stamped_write_with_changes(Stamp::new(2))?;
    assert_eq!(vec.len(), 5);
    assert_eq!(vec.collect(), vec![0, 1, 2, 100, 200]);

    vec.rollback()?;
    assert_eq!(vec.len(), 5, "len mismatch after rollback");
    assert_eq!(
        vec.collect(),
        vec![0, 1, 2, 3, 4],
        "data mismatch after rollback"
    );
    assert_eq!(vec.stamp(), Stamp::new(1));

    Ok(())
}

fn run_rollback_truncate_then_reflush<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = import_with_changes::<V>(&db, "truncate_reflush", 10)?;

    for i in 0..10 {
        vec.push(i);
    }
    vec.stamped_write_with_changes(Stamp::new(1))?;

    vec.truncate_if_needed_at(4)?;
    vec.stamped_write_with_changes(Stamp::new(2))?;
    assert_eq!(vec.len(), 4);

    vec.rollback()?;
    assert_eq!(vec.len(), 10);
    assert_eq!(vec.collect(), (0..10).collect::<Vec<u32>>());

    vec.stamped_write_with_changes(Stamp::new(2))?;
    assert_eq!(vec.len(), 10);
    assert_eq!(vec.stored_len(), 10);
    assert_eq!(vec.collect(), (0..10).collect::<Vec<u32>>());

    Ok(())
}

fn run_rollback_truncate_persistence<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    {
        let mut vec = import_with_changes::<V>(&db, "persist", 10)?;
        for i in 0..7 {
            vec.push(i);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;

        vec.truncate_if_needed_at(2)?;
        vec.stamped_write_with_changes(Stamp::new(2))?;

        vec.rollback()?;
        vec.stamped_write_with_changes(Stamp::new(1))?;
    }
    {
        let vec = import_with_changes::<V>(&db, "persist", 10)?;
        assert_eq!(vec.len(), 7);
        assert_eq!(vec.collect(), (0..7).collect::<Vec<u32>>());
        assert_eq!(vec.stamp(), Stamp::new(1));
    }
    Ok(())
}

fn run_malformed_rollback_preserves_state<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = import_with_changes::<V>(&db, "malformed", 10)?;
    for value in 0..8 {
        vec.push(value);
    }
    vec.stamped_write_with_changes(Stamp::new(1))?;
    vec.truncate_if_needed_at(3)?;
    vec.push(100);
    vec.stamped_write_with_changes(Stamp::new(2))?;
    let files = vec.find_rollback_files()?;
    let path = &files[&Stamp::new(2)];
    let original = fs::read(path)?;
    // The append-only change-file prefix is shared by all formats, including
    // mutable wrappers. Check its exact persisted representation.
    let mut expected = Vec::new();
    for value in [1_u64, 8, 3, 5] {
        expected.extend(value.to_le_bytes());
    }
    for value in 3_u32..8 {
        expected.extend(value.to_le_bytes());
    }
    expected.extend(0_u64.to_le_bytes()); // previous pushed count
    expected.extend(0_u64.to_le_bytes()); // newly appended values need no undo
    assert_eq!(&original[..expected.len()], expected);
    for length in 0..original.len() {
        fs::write(path, &original[..length])?;
        assert!(vec.rollback().is_err(), "accepted {length}-byte prefix");
        assert_eq!(vec.stamp(), Stamp::new(2));
        assert_eq!(vec.stored_len(), 4);
        assert!(vec.pushed().is_empty());
        assert_eq!(vec.collect(), [0, 1, 2, 100]);
    }
    fs::write(path, original)?;
    vec.rollback()?;
    assert_eq!(vec.stamp(), Stamp::new(1));
    assert_eq!(vec.collect(), (0..8).collect::<Vec<_>>());
    vec.write()?;
    drop(vec);
    let vec = import_with_changes::<V>(&db, "malformed", 10)?;
    assert_eq!(vec.collect(), (0..8).collect::<Vec<_>>());
    Ok(())
}

macro_rules! instantiate_for {
    ($mod:ident, $ty:ty) => {
        mod $mod {
            use super::*;
            type V = $ty;

            #[test]
            fn malformed_rollback_preserves_state() -> vecdb::Result<()> {
                run_malformed_rollback_preserves_state::<V>()
            }

            #[test]
            fn rollback_after_pure_truncate() -> vecdb::Result<()> {
                run_rollback_after_pure_truncate::<V>()
            }
            #[test]
            fn rollback_after_truncate_and_push() -> vecdb::Result<()> {
                run_rollback_after_truncate_and_push::<V>()
            }
            #[test]
            fn rollback_truncate_then_reflush() -> vecdb::Result<()> {
                run_rollback_truncate_then_reflush::<V>()
            }
            #[test]
            fn rollback_truncate_persistence() -> vecdb::Result<()> {
                run_rollback_truncate_persistence::<V>()
            }
        }
    };
}

instantiate_for!(bytes, vecdb::BytesVec<usize, u32>);

instantiate_for!(
    mutable_bytes,
    vecdb::MutableVec<vecdb::BytesVec<usize, u32>>
);

#[cfg(feature = "zerocopy")]
instantiate_for!(
    mutable_zerocopy,
    vecdb::MutableVec<vecdb::ZeroCopyVec<usize, u32>>
);

#[cfg(feature = "zerocopy")]
instantiate_for!(zerocopy, vecdb::ZeroCopyVec<usize, u32>);

#[cfg(feature = "pco")]
instantiate_for!(pco, vecdb::PcoVec<usize, u32>);

#[cfg(feature = "lz4")]
instantiate_for!(lz4, vecdb::LZ4Vec<usize, u32>);

#[cfg(feature = "zstd")]
instantiate_for!(zstd, vecdb::ZstdVec<usize, u32>);
