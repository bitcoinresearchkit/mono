//! Compressed page boundaries, raw-tail appends, truncation, and reopening.

use rawdb::Database;
use tempfile::TempDir;
#[cfg(feature = "lz4")]
use vecdb::LZ4Vec;
#[cfg(feature = "pco")]
use vecdb::PcoVec;
#[cfg(feature = "zstd")]
use vecdb::ZstdVec;
use vecdb::{ReadableVec, Result, StoredVec, Version};

const PER_PAGE_U32: usize = 8 * 1024 / size_of::<u32>();

fn setup_db() -> Result<(Database, TempDir)> {
    let temp = TempDir::new()?;
    let db = Database::open(temp.path())?;
    Ok((db, temp))
}

// Small writes stay raw, data survives reopen

fn test_small_write_raw_survives_reopen<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let values: Vec<u32> = (0..100).collect();

    {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        for &v in &values {
            vec.push(v);
        }
        vec.write()?;
        assert_eq!(vec.stored_len(), 100);
        assert_eq!(vec.collect(), values);
    }

    // Reopen and verify
    {
        let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        assert_eq!(vec.stored_len(), 100);
        assert_eq!(vec.collect(), values);
        assert_eq!(vec.collect_range(0, 1), vec![0]);
        assert_eq!(vec.collect_range(50, 53), vec![50, 51, 52]);
        assert_eq!(vec.collect_range(99, 100), vec![99]);
    }

    Ok(())
}

// Fast-append path: multiple small writes to same raw page

fn test_fast_append_multiple_small_writes<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;

    // Write 10 values, flush → raw page
    for v in 0..10u32 {
        vec.push(v);
    }
    vec.write()?;
    assert_eq!(vec.stored_len(), 10);

    // Write 10 more → fast append (raw page, room left)
    for v in 10..20u32 {
        vec.push(v);
    }
    vec.write()?;
    assert_eq!(vec.stored_len(), 20);
    assert_eq!(vec.collect(), (0..20).collect::<Vec<u32>>());

    // Write 10 more again → fast append
    for v in 20..30u32 {
        vec.push(v);
    }
    vec.write()?;
    assert_eq!(vec.stored_len(), 30);
    assert_eq!(vec.collect(), (0..30).collect::<Vec<u32>>());

    // Range reads across appended data
    assert_eq!(vec.collect_range(5, 15), (5..15).collect::<Vec<u32>>());
    assert_eq!(vec.collect_range(15, 25), (15..25).collect::<Vec<u32>>());

    Ok(())
}

// Fast-append survives reopen and continued appending

fn test_fast_append_survives_reopen<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        for v in 0..50u32 {
            vec.push(v);
        }
        vec.write()?;

        // Fast append
        for v in 50..100u32 {
            vec.push(v);
        }
        vec.write()?;
        assert_eq!(vec.collect(), (0..100).collect::<Vec<u32>>());
    }

    // Reopen and verify the raw page was persisted correctly
    {
        let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        assert_eq!(vec.stored_len(), 100);
        assert_eq!(vec.collect(), (0..100).collect::<Vec<u32>>());
    }

    // Reopen and continue appending (should fast-append to existing raw page)
    {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        for v in 100..150u32 {
            vec.push(v);
        }
        vec.write()?;
        assert_eq!(vec.collect(), (0..150).collect::<Vec<u32>>());
    }

    // Final reopen
    {
        let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        assert_eq!(vec.collect(), (0..150).collect::<Vec<u32>>());
    }

    Ok(())
}

// Full page gets compressed, partial tail stays raw

fn test_full_page_compressed_partial_raw<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let count = PER_PAGE_U32 + 100;
    let values: Vec<u32> = (0..count as u32).collect();

    {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        for &v in &values {
            vec.push(v);
        }
        vec.write()?;
        assert_eq!(vec.stored_len(), count);
        assert_eq!(vec.collect(), values);

        // Read across compressed→raw page boundary
        let b = PER_PAGE_U32;
        assert_eq!(
            vec.collect_range(b - 2, b + 2),
            vec![(b - 2) as u32, (b - 1) as u32, b as u32, (b + 1) as u32]
        );
    }

    // Reopen and verify both compressed and raw pages survive
    {
        let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        assert_eq!(vec.stored_len(), count);
        assert_eq!(vec.collect(), values);
    }

    Ok(())
}

// Exact page boundary — no raw tail

fn test_exact_page_boundary<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let count = PER_PAGE_U32;
    let values: Vec<u32> = (0..count as u32).collect();

    {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        for &v in &values {
            vec.push(v);
        }
        vec.write()?;
        assert_eq!(vec.stored_len(), count);
        assert_eq!(vec.collect(), values);
    }

    // Reopen
    {
        let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        assert_eq!(vec.stored_len(), count);
        assert_eq!(vec.collect(), values);
    }

    // Append a few more → creates a new raw page
    {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        for v in count as u32..(count + 5) as u32 {
            vec.push(v);
        }
        vec.write()?;
        assert_eq!(vec.collect(), (0..(count + 5) as u32).collect::<Vec<u32>>());
    }

    // Reopen and verify
    {
        let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        assert_eq!(vec.collect(), (0..(count + 5) as u32).collect::<Vec<u32>>());
    }

    Ok(())
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

// Fast-append fills exactly to page boundary

fn test_fast_append_fills_exactly<V>() -> Result<()>
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

    // Push exactly 10 more → fills page exactly
    // fast-append condition is `partial_len + pushed_len < PER_PAGE` (strict <),
    // so this goes through the normal path and compresses the full page
    for v in initial as u32..PER_PAGE_U32 as u32 {
        vec.push(v);
    }
    vec.write()?;

    assert_eq!(vec.stored_len(), PER_PAGE_U32);
    assert_eq!(
        vec.collect(),
        (0..PER_PAGE_U32 as u32).collect::<Vec<u32>>()
    );

    // Append more → new raw page
    for v in PER_PAGE_U32 as u32..(PER_PAGE_U32 + 5) as u32 {
        vec.push(v);
    }
    vec.write()?;
    assert_eq!(
        vec.collect(),
        (0..(PER_PAGE_U32 + 5) as u32).collect::<Vec<u32>>()
    );

    Ok(())
}

// Incremental growth across multiple pages

fn test_incremental_growth_across_pages<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;

    let chunk_size = 1000;
    let total_target = PER_PAGE_U32 * 2 + 500;
    let num_chunks = total_target.div_ceil(chunk_size);
    let total = num_chunks * chunk_size;

    for chunk_i in 0..num_chunks {
        let start = chunk_i * chunk_size;
        for v in start..(start + chunk_size) {
            vec.push(v as u32);
        }
        vec.write()?;

        let expected: Vec<u32> = (0..(start + chunk_size) as u32).collect();
        assert_eq!(vec.collect(), expected, "Mismatch after chunk {}", chunk_i);
    }

    // Final full verification
    assert_eq!(vec.collect(), (0..total as u32).collect::<Vec<u32>>());

    // Reads at page boundaries
    assert_eq!(vec.collect_range(0, 10), (0..10).collect::<Vec<u32>>());
    assert_eq!(
        vec.collect_range(PER_PAGE_U32 - 5, PER_PAGE_U32 + 5),
        ((PER_PAGE_U32 - 5) as u32..(PER_PAGE_U32 + 5) as u32).collect::<Vec<u32>>()
    );
    assert_eq!(
        vec.collect_range(PER_PAGE_U32 * 2 - 5, PER_PAGE_U32 * 2 + 5),
        ((PER_PAGE_U32 * 2 - 5) as u32..(PER_PAGE_U32 * 2 + 5) as u32).collect::<Vec<u32>>()
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

// Reset clears raw pages

fn test_reset_clears_raw_pages<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;

    for v in 0..100u32 {
        vec.push(v);
    }
    vec.write()?;
    assert_eq!(vec.stored_len(), 100);

    vec.reset()?;
    assert_eq!(vec.stored_len(), 0);
    assert_eq!(vec.len(), 0);
    assert!(vec.collect().is_empty());

    // Write new data after reset
    for v in 1000..1050u32 {
        vec.push(v);
    }
    vec.write()?;
    assert_eq!(vec.collect(), (1000..1050).collect::<Vec<u32>>());

    Ok(())
}

// Reset after multi-page data (compressed + raw)

fn test_reset_after_multi_page<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;

    {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        let count = PER_PAGE_U32 + 200;
        for v in 0..count as u32 {
            vec.push(v);
        }
        vec.write()?;

        vec.reset()?;
        assert_eq!(vec.len(), 0);
        assert!(vec.collect().is_empty());

        // Write again
        for v in 0..50u32 {
            vec.push(v);
        }
        vec.write()?;
        assert_eq!(vec.collect(), (0..50).collect::<Vec<u32>>());
    }

    // Reopen
    {
        let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        assert_eq!(vec.collect(), (0..50).collect::<Vec<u32>>());
    }

    Ok(())
}

// Read spanning compressed and raw pages

fn test_read_spanning_compressed_and_raw<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;

    let count = PER_PAGE_U32 + 200;
    for v in 0..count as u32 {
        vec.push(v);
    }
    vec.write()?;

    // Spanning compressed→raw boundary
    let from = PER_PAGE_U32 - 50;
    let to = PER_PAGE_U32 + 50;
    assert_eq!(
        vec.collect_range(from, to),
        (from as u32..to as u32).collect::<Vec<u32>>()
    );

    // Entirely within compressed page
    assert_eq!(vec.collect_range(0, 100), (0..100).collect::<Vec<u32>>());

    // Entirely within raw page
    assert_eq!(
        vec.collect_range(PER_PAGE_U32, PER_PAGE_U32 + 100),
        (PER_PAGE_U32 as u32..(PER_PAGE_U32 + 100) as u32).collect::<Vec<u32>>()
    );

    // Full read
    assert_eq!(vec.collect(), (0..count as u32).collect::<Vec<u32>>());

    Ok(())
}

// Multiple pages with raw tail

fn test_multiple_pages_with_raw_tail<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;

    // 3 full pages + partial raw tail
    let count = PER_PAGE_U32 * 3 + 777;
    for v in 0..count as u32 {
        vec.push(v);
    }
    vec.write()?;
    assert_eq!(vec.stored_len(), count);

    // Read from each page
    for page in 0..4 {
        let start = page * PER_PAGE_U32;
        let end = (start + 10).min(count);
        let result = vec.collect_range(start, end);
        let expected: Vec<u32> = (start as u32..end as u32).collect();
        assert_eq!(result, expected, "Page {} read mismatch", page);
    }

    // Read spanning each page boundary
    for boundary_page in 1..=3 {
        let mid = boundary_page * PER_PAGE_U32;
        let from = mid - 5;
        let to = (mid + 5).min(count);
        assert_eq!(
            vec.collect_range(from, to),
            (from as u32..to as u32).collect::<Vec<u32>>(),
            "Boundary {} read mismatch",
            boundary_page
        );
    }

    // Full read
    assert_eq!(vec.collect(), (0..count as u32).collect::<Vec<u32>>());

    Ok(())
}

// Write-reopen-append cycle (simulates real usage patterns)

fn test_write_reopen_append_cycle<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;

    let mut total = 0u32;
    for cycle in 0..20u32 {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        let count = 100 + cycle * 50;
        for _ in 0..count {
            vec.push(total);
            total += 1;
        }
        vec.write()?;
        assert_eq!(
            vec.collect(),
            (0..total).collect::<Vec<u32>>(),
            "Cycle {}",
            cycle
        );
    }

    // Final verification after all cycles
    {
        let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        assert_eq!(vec.stored_len(), total as usize);
        assert_eq!(vec.collect(), (0..total).collect::<Vec<u32>>());
    }

    Ok(())
}

// Write-reopen-append cycle crossing page boundaries

fn test_write_reopen_cycle_crossing_pages<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;

    // Use a chunk size that doesn't align with PER_PAGE to ensure we cross boundaries
    let chunk = PER_PAGE_U32 / 3 + 7;
    let mut total = 0u32;
    for cycle in 0..10u32 {
        let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        for _ in 0..chunk {
            vec.push(total);
            total += 1;
        }
        vec.write()?;
        assert_eq!(
            vec.collect(),
            (0..total).collect::<Vec<u32>>(),
            "Cycle {} (total={})",
            cycle,
            total
        );
    }

    {
        let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
        assert_eq!(vec.collect(), (0..total).collect::<Vec<u32>>());
    }

    Ok(())
}

fn test_noop_write<V>(count: usize) -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;

    for v in 0..count as u32 {
        vec.push(v);
    }
    vec.write()?;

    let changed = vec.write()?;
    assert!(!changed);
    assert_eq!(vec.collect(), (0..count as u32).collect::<Vec<u32>>());

    Ok(())
}

// fold/iteration over mixed compressed+raw pages

fn test_fold_over_mixed_pages<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;

    let count = PER_PAGE_U32 + 500;
    for v in 0..count as u32 {
        vec.push(v);
    }
    vec.write()?;

    // Sum all values using for_each
    let expected_sum: u64 = (0..count as u64).sum();
    let mut actual_sum = 0u64;
    vec.for_each(|v: u32| actual_sum += v as u64);
    assert_eq!(actual_sum, expected_sum);

    // fold_range spanning the compressed→raw boundary
    let from = PER_PAGE_U32 - 100;
    let to = PER_PAGE_U32 + 100;
    let range_sum = vec.fold_range_at(from, to, 0u64, |acc, v: u32| acc + v as u64);
    let expected_range_sum: u64 = (from as u64..to as u64).sum();
    assert_eq!(range_sum, expected_range_sum);

    Ok(())
}

// Pushed (un-flushed) values mixed with stored raw page

fn test_pushed_and_stored_raw_page<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;

    // Write some values → stored as raw page
    for v in 0..100u32 {
        vec.push(v);
    }
    vec.write()?;

    // Push more without writing → pushed buffer
    for v in 100..150u32 {
        vec.push(v);
    }

    assert_eq!(vec.stored_len(), 100);
    assert_eq!(vec.pushed_len(), 50);
    assert_eq!(vec.len(), 150);

    // Read spanning stored raw + pushed
    assert_eq!(vec.collect_range(90, 110), (90..110).collect::<Vec<u32>>());
    assert_eq!(vec.collect(), (0..150).collect::<Vec<u32>>());

    // Now write
    vec.write()?;
    assert_eq!(vec.stored_len(), 150);
    assert_eq!(vec.pushed_len(), 0);
    assert_eq!(vec.collect(), (0..150).collect::<Vec<u32>>());

    Ok(())
}

// Pushed values mixed with stored compressed + raw pages

fn test_pushed_and_stored_mixed_pages<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;

    let count = PER_PAGE_U32 + 100;
    for v in 0..count as u32 {
        vec.push(v);
    }
    vec.write()?;

    // Push more without writing
    for v in count as u32..(count + 50) as u32 {
        vec.push(v);
    }

    assert_eq!(vec.stored_len(), count);
    assert_eq!(vec.pushed_len(), 50);
    assert_eq!(vec.len(), count + 50);

    // Read spanning raw page → pushed buffer
    assert_eq!(
        vec.collect_range(count - 10, count + 10),
        ((count - 10) as u32..(count + 10) as u32).collect::<Vec<u32>>()
    );

    // Read spanning compressed → raw → pushed
    let from = PER_PAGE_U32 - 5;
    let to = count + 5;
    assert_eq!(
        vec.collect_range(from, to),
        (from as u32..to as u32).collect::<Vec<u32>>()
    );

    Ok(())
}

// Single value writes (extreme fast-append case)

fn test_single_value_writes<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;

    // Push and write one value at a time, 50 times
    for v in 0..50u32 {
        vec.push(v);
        vec.write()?;
    }

    assert_eq!(vec.stored_len(), 50);
    assert_eq!(vec.collect(), (0..50).collect::<Vec<u32>>());

    // Reopen
    drop(vec);
    let vec: V = V::forced_import(&db, "vec", Version::TWO)?;
    assert_eq!(vec.collect(), (0..50).collect::<Vec<u32>>());

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

// Read-only clone reads mixed pages correctly

fn test_read_only_clone_mixed_pages<V>() -> Result<()>
where
    V: StoredVec<I = usize, T = u32>,
{
    let (db, _tmp) = setup_db()?;
    let mut vec: V = V::forced_import(&db, "vec", Version::TWO)?;

    let count = PER_PAGE_U32 + 200;
    for v in 0..count as u32 {
        vec.push(v);
    }
    vec.write()?;

    let ro = vec.read_only_clone();
    assert_eq!(ro.collect(), (0..count as u32).collect::<Vec<u32>>());

    // Read across boundary via read-only clone
    let b = PER_PAGE_U32;
    assert_eq!(
        ro.collect_range(b - 5, b + 5),
        ((b - 5) as u32..(b + 5) as u32).collect::<Vec<u32>>()
    );

    Ok(())
}

// Test instantiation for each compression strategy

fn page_cases<V: StoredVec<I = usize, T = u32>>() -> Result<()> {
    test_small_write_raw_survives_reopen::<V>()?;
    test_fast_append_multiple_small_writes::<V>()?;
    test_fast_append_survives_reopen::<V>()?;
    test_full_page_compressed_partial_raw::<V>()?;
    test_exact_page_boundary::<V>()?;
    test_fast_append_overflow::<V>()?;
    test_fast_append_fills_exactly::<V>()?;
    test_incremental_growth_across_pages::<V>()?;
    // Truncate within the raw tail, at the page boundary, and within a compressed page.
    test_truncation_case::<V>(PER_PAGE_U32 + 200, 50)?;
    test_truncation_case::<V>(PER_PAGE_U32, 10)?;
    test_truncation_case::<V>(PER_PAGE_U32 / 2, 100)?;
    test_reset_clears_raw_pages::<V>()?;
    test_reset_after_multi_page::<V>()?;
    test_read_spanning_compressed_and_raw::<V>()?;
    test_multiple_pages_with_raw_tail::<V>()?;
    test_write_reopen_append_cycle::<V>()?;
    test_write_reopen_cycle_crossing_pages::<V>()?;
    // A second write stays a no-op for both one-page and multi-page data.
    test_noop_write::<V>(50)?;
    test_noop_write::<V>(PER_PAGE_U32 + 100)?;
    test_fold_over_mixed_pages::<V>()?;
    test_pushed_and_stored_raw_page::<V>()?;
    test_pushed_and_stored_mixed_pages::<V>()?;
    test_single_value_writes::<V>()?;
    test_truncate_to_zero_then_rebuild::<V>()?;
    test_read_only_clone_mixed_pages::<V>()?;
    Ok(())
}

#[cfg(feature = "pco")]
#[test]
fn pco_pages() -> Result<()> {
    page_cases::<PcoVec<usize, u32>>()
}

#[cfg(feature = "lz4")]
#[test]
fn lz4_pages() -> Result<()> {
    page_cases::<LZ4Vec<usize, u32>>()
}

#[cfg(feature = "zstd")]
#[test]
fn zstd_pages() -> Result<()> {
    page_cases::<ZstdVec<usize, u32>>()
}
