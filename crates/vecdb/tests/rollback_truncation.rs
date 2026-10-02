//! Truncation rollback across native writes and reopening.

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

macro_rules! instantiate_for {
    ($mod:ident, $ty:ty) => {
        mod $mod {
            use super::*;
            type V = $ty;

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

#[cfg(feature = "pco")]
instantiate_for!(pco, vecdb::PcoVec<usize, u32>);
