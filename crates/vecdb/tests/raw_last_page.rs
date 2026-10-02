//! Compressed page boundaries, raw-tail appends, truncation, and reopening.

use rawdb::Database;
use tempfile::TempDir;
#[cfg(feature = "pco")]
use vecdb::PcoVec;
use vecdb::{Result, StoredVec, Version};

const PER_PAGE_U32: usize = 8 * 1024 / size_of::<u32>();

fn setup_db() -> Result<(Database, TempDir)> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    Ok((db, temp))
}

// Fast-append overflow: fills the raw page, triggers compression

fn test_fast_append_overflow<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;

    // Write almost a full page
    let initial = PER_PAGE_U32 - 10;
    for v in 0..initial as u32 {
        vec.push(v);
    }
    vec.write()?;
    assert_eq!(vec.stored_len(), initial);

    // Push 20 more → exceeds page capacity, can't fast-append
    // Should go through normal path: compress full page, raw tail of 10
    for v in initial as u32..(initial + 20) as u32 {
        vec.push(v);
    }
    vec.write()?;

    let total = initial + 20;
    let expected: Vec<u32> = (0..total as u32).collect();
    assert_eq!(vec.stored_len(), total);
    assert_eq!(vec.collect(), expected);

    // Verify reads across compressed/raw boundary
    let b = PER_PAGE_U32;
    assert_eq!(
        vec.collect_range(b - 2, b + 2),
        vec![(b - 2) as u32, (b - 1) as u32, b as u32, (b + 1) as u32]
    );

    Ok(())
}

fn test_truncation_case<V>(truncate_to: usize, append_count: usize) -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;

    {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        let count = PER_PAGE_U32 + 500;
        for v in 0..count as u32 {
            vec.push(v);
        }
        vec.write()?;

        vec.truncate_if_needed(truncate_to)?;
        vec.write()?;
        assert_eq!(vec.collect(), (0..truncate_to as u32).collect::<Vec<u32>>());
    }

    {
        let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        assert_eq!(vec.stored_len(), truncate_to);
        assert_eq!(vec.collect(), (0..truncate_to as u32).collect::<Vec<u32>>());
    }

    {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        for v in truncate_to as u32..(truncate_to + append_count) as u32 {
            vec.push(v);
        }
        vec.write()?;
        assert_eq!(
            vec.collect(),
            (0..(truncate_to + append_count) as u32).collect::<Vec<u32>>()
        );
    }

    Ok(())
}

// Reopen after each append, including batches that cross page boundaries.

fn test_write_reopen_append_cycle<V>(counts: impl IntoIterator<Item = usize>) -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut total = 0u32;
    for (cycle, count) in counts.into_iter().enumerate() {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        for _ in 0..count {
            vec.push(total);
            total += 1;
        }
        vec.write()?;
        assert_eq!(
            vec.collect(),
            (0..total).collect::<Vec<u32>>(),
            "Cycle {cycle}"
        );
    }
    let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
    assert_eq!(vec.stored_len(), total as usize);
    assert_eq!(vec.collect(), (0..total).collect::<Vec<u32>>());
    Ok(())
}

// Truncate all then rebuild (edge case)

fn test_truncate_to_zero_then_rebuild<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;

    {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        let count = PER_PAGE_U32 + 100;
        for v in 0..count as u32 {
            vec.push(v);
        }
        vec.write()?;

        vec.truncate_if_needed(0)?;
        assert_eq!(vec.len(), 0);

        // Rebuild with different data
        for v in 5000..5100u32 {
            vec.push(v);
        }
        vec.write()?;
        assert_eq!(vec.collect(), (5000..5100).collect::<Vec<u32>>());
    }

    // Reopen
    {
        let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        assert_eq!(vec.collect(), (5000..5100).collect::<Vec<u32>>());
    }

    Ok(())
}

// Test instantiation for each compression strategy

fn page_cases<V: StoredVec<I = usize, T = u32>>() -> Result<()> {
    test_fast_append_overflow::<V>()?;

    // Truncate within the raw tail, at the page boundary, and within a compressed page.
    test_truncation_case::<V>(PER_PAGE_U32 + 200, 50)?;
    test_truncation_case::<V>(PER_PAGE_U32, 10)?;
    test_truncation_case::<V>(PER_PAGE_U32 / 2, 100)?;

    test_write_reopen_append_cycle::<V>((0..20usize).map(|cycle| 100 + cycle * 50))?;
    test_write_reopen_append_cycle::<V>((0..10).map(|_| PER_PAGE_U32 / 3 + 7))?;
    test_truncate_to_zero_then_rebuild::<V>()?;
    Ok(())
}

#[cfg(feature = "pco")]
#[test]
fn pco_pages() -> Result<()> {
    page_cases::<PcoVec<usize, u32>>()
}
