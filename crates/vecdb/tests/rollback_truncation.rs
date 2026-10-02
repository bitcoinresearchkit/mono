//! Truncation rollback, reflush/reopen, and malformed change-file validation.

use std::fs;

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{ImportOptions, Result, Stamp, StoredVec, Version};

fn setup_db() -> Result<(Database, TempDir)> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    Ok((db, temp))
}

fn import_with_changes<V>(db: &Database, name: &str) -> Result<V>
where
    V: StoredVec<I = usize, T = u32>,
{
    let mut options: ImportOptions = (db, name, Version::TWO).into();
    options = options.with_saved_stamped_changes(10);
    V::forced_import_with(options)
}

fn run_truncate_rollback_and_reopen<V: StoredVec<I = usize, T = u32>>() -> Result<()> {
    for (count, truncate_to, replacement) in [(8, 5, &[][..]), (5, 3, &[100, 200][..])] {
        let (db, _temp) = setup_db()?;
        let mut vec = import_with_changes::<V>(&db, "values")?;
        for value in 0..count {
            vec.push(value);
        }
        vec.stamped_write_with_changes(Stamp::new(1))?;
        vec.truncate_if_needed_at(truncate_to as usize)?;
        for &value in replacement {
            vec.push(value);
        }
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(
            vec.collect(),
            (0..truncate_to)
                .chain(replacement.iter().copied())
                .collect::<Vec<_>>()
        );
        assert_eq!(vec.rollback_before(Stamp::new(2))?, Stamp::new(1));
        assert_eq!(vec.len(), count as usize);
        assert_eq!(vec.collect(), (0..count).collect::<Vec<_>>());
        assert_eq!(vec.stamp(), Stamp::new(1));

        for stamp in [Stamp::new(1), Stamp::new(2)] {
            vec.stamped_write(stamp)?;
            assert_eq!(vec.stored_len(), count as usize);
            assert_eq!(vec.collect(), (0..count).collect::<Vec<_>>());
            drop(vec);
            vec = import_with_changes::<V>(&db, "values")?;
            assert_eq!(vec.stamp(), stamp);
            assert_eq!(vec.stored_len(), count as usize);
            assert_eq!(vec.collect(), (0..count).collect::<Vec<_>>());
        }
    }
    Ok(())
}

fn run_malformed_rollback_preserves_state<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _temp) = setup_db()?;
    let mut vec = import_with_changes::<V>(&db, "malformed")?;
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
    let vec = import_with_changes::<V>(&db, "malformed")?;
    assert_eq!(vec.collect(), (0..8).collect::<Vec<_>>());
    Ok(())
}

macro_rules! instantiate_for {
    ($mod:ident, $ty:ty) => {
        mod $mod {
            use super::*;
            type V = $ty;

            #[test]
            fn malformed_rollback_preserves_state() -> Result<()> {
                run_malformed_rollback_preserves_state::<V>()
            }

            #[test]
            fn truncate_rollback_reflush_and_reopen() -> Result<()> {
                run_truncate_rollback_and_reopen::<V>()
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
