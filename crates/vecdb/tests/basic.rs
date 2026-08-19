use rawdb::Database;
use tempfile::TempDir;
use vecdb::{Stamp, StoredVec, Version};

/// Helper to create a temporary test database
pub fn setup_test_db() -> vecdb::Result<(Database, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db = Database::open(temp_dir.path())?;
    Ok((db, temp_dir))
}

/// Generic test function for basic vec operations
fn run_vec_operations<V>() -> Result<(), Box<dyn std::error::Error>>
where
    V: StoredVec<I = usize, T = u32>,
{
    let version = Version::TWO;
    let (database, _temp) = setup_test_db()?;
    let options = (&database, "vec", version).into();

    {
        let mut vec: V = V::forced_import_with(options)?;

        (0..21_u32).for_each(|v| {
            vec.push(v);
        });

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(1, 2), vec![1]);
        assert_eq!(vec.collect_range(2, 3), vec![2]);
        assert_eq!(vec.collect_range(20, 21), vec![20]);
        assert!(vec.collect_range(21, 22).is_empty());

        vec.write()?;

        assert_eq!(vec.header().stamp(), Stamp::new(0));
    }

    {
        let mut vec: V = V::forced_import_with(options)?;

        vec.mut_header().update_stamp(Stamp::new(100));

        assert_eq!(vec.header().stamp(), Stamp::new(100));

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(1, 2), vec![1]);
        assert_eq!(vec.collect_range(2, 3), vec![2]);
        assert_eq!(vec.collect_range(3, 4), vec![3]);
        assert_eq!(vec.collect_range(4, 5), vec![4]);
        assert_eq!(vec.collect_range(5, 6), vec![5]);
        assert_eq!(vec.collect_range(20, 21), vec![20]);
        assert_eq!(vec.collect_range(0, 1), vec![0]);

        vec.push(21);
        vec.push(22);

        assert_eq!(vec.stored_len(), 21);
        assert_eq!(vec.pushed_len(), 2);
        assert_eq!(vec.len(), 23);

        assert_eq!(vec.collect_range(20, 21), vec![20]);
        assert_eq!(vec.collect_range(21, 22), vec![21]);
        assert_eq!(vec.collect_range(22, 23), vec![22]);
        assert!(vec.collect_range(23, 24).is_empty());

        vec.write()?;
    }

    {
        let mut vec: V = V::forced_import_with(options)?;

        assert_eq!(vec.header().stamp(), Stamp::new(100));

        assert_eq!(vec.stored_len(), 23);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.len(), 23);

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(20, 21), vec![20]);
        assert_eq!(vec.collect_range(21, 22), vec![21]);
        assert_eq!(vec.collect_range(22, 23), vec![22]);

        vec.truncate_if_needed(14)?;

        assert_eq!(vec.stored_len(), 14);
        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.len(), 14);

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(5, 6), vec![5]);
        assert!(vec.collect_range(20, 21).is_empty());

        assert_eq!(
            vec.collect_signed_range(Some(-5), None),
            vec![9, 10, 11, 12, 13]
        );

        vec.push(vec.len() as u32);
        let last = vec.collect_range(vec.len() - 1, vec.len());
        assert_eq!(last[0], 14);

        vec.write()?;

        assert_eq!(
            vec.collect(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        );
    }

    {
        let mut vec: V = V::forced_import_with(options)?;

        assert_eq!(
            vec.collect(),
            vec![0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14]
        );

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(5, 6), vec![5]);
        assert!(vec.collect_range(20, 21).is_empty());

        assert_eq!(
            vec.collect_signed_range(Some(-5), None),
            vec![10, 11, 12, 13, 14]
        );

        vec.reset()?;

        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.len(), 0);

        (0..21_u32).for_each(|v| {
            vec.push(v);
        });

        assert_eq!(vec.pushed_len(), 21);
        assert_eq!(vec.stored_len(), 0);
        assert_eq!(vec.len(), 21);

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(20, 21), vec![20]);
        assert!(vec.collect_range(21, 22).is_empty());

        vec.write()?;
    }

    {
        let mut vec: V = V::forced_import_with(options)?;

        assert_eq!(vec.pushed_len(), 0);
        assert_eq!(vec.stored_len(), 21);
        assert_eq!(vec.len(), 21);

        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(10, 11), vec![10]);

        vec.write()?;
    }

    {
        let vec: V = V::forced_import_with(options)?;

        assert_eq!(
            vec.collect(),
            vec![
                0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20,
            ]
        );
    }

    Ok(())
}

// ============================================================================
// Test instantiation for each vec type
// ============================================================================

mod bytes {
    use super::*;
    use vecdb::BytesVec;
    type V = BytesVec<usize, u32>;

    #[test]
    fn test_vec_operations() -> Result<(), Box<dyn std::error::Error>> {
        run_vec_operations::<V>()
    }
}

#[cfg(feature = "zerocopy")]
mod zerocopy {
    use super::*;
    use vecdb::ZeroCopyVec;
    type V = ZeroCopyVec<usize, u32>;

    #[test]
    fn test_vec_operations() -> Result<(), Box<dyn std::error::Error>> {
        run_vec_operations::<V>()
    }
}

#[cfg(feature = "pco")]
mod pco {
    use super::*;
    use vecdb::PcoVec;
    type V = PcoVec<usize, u32>;

    #[test]
    fn test_vec_operations() -> Result<(), Box<dyn std::error::Error>> {
        run_vec_operations::<V>()
    }
}

#[cfg(feature = "lz4")]
mod lz4 {
    use super::*;
    use vecdb::LZ4Vec;
    type V = LZ4Vec<usize, u32>;

    #[test]
    fn test_vec_operations() -> Result<(), Box<dyn std::error::Error>> {
        run_vec_operations::<V>()
    }
}

#[cfg(feature = "zstd")]
mod zstd {
    use super::*;
    use vecdb::ZstdVec;
    type V = ZstdVec<usize, u32>;

    #[test]
    fn test_vec_operations() -> Result<(), Box<dyn std::error::Error>> {
        run_vec_operations::<V>()
    }
}

#[cfg(feature = "zerocopy")]
mod eager_zerocopy {
    use super::*;
    use vecdb::{EagerVec, ZeroCopyVec};
    type V = EagerVec<ZeroCopyVec<usize, u32>>;

    #[test]
    fn test_vec_operations() -> Result<(), Box<dyn std::error::Error>> {
        run_vec_operations::<V>()
    }
}

#[cfg(feature = "pco")]
mod eager_pco {
    use super::*;
    use vecdb::{EagerVec, PcoVec};
    type V = EagerVec<PcoVec<usize, u32>>;

    #[test]
    fn test_vec_operations() -> Result<(), Box<dyn std::error::Error>> {
        run_vec_operations::<V>()
    }
}
