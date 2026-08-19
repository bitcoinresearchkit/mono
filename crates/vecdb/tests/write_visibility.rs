//! Tests that write() makes data visible to other vecs without fsync.
//!
//! This verifies the key property that enables fast per-compute operations:
//! after vec_a.write(), vec_b can immediately read the new data because
//! both share the same mmap. No fsync is needed for in-process visibility.

use rawdb::Database;
use tempfile::TempDir;
use vecdb::{StoredVec, Version};

fn setup_test_db() -> vecdb::Result<(Database, TempDir)> {
    let temp_dir = TempDir::new()?;
    let db = Database::open(temp_dir.path())?;
    Ok((db, temp_dir))
}

/// Tests that after write() on vec_a, a separate vec_b instance
/// can read the new data (simulating derived vec computation).
fn run_write_visibility_test<V>() -> Result<(), Box<dyn std::error::Error>>
where
    V: StoredVec<I = usize, T = u32>,
{
    let version = Version::ZERO;
    let (database, _temp) = setup_test_db()?;

    // Create and populate vec_a
    let mut vec_a: V = V::forced_import(&database, "vec_a", version)?;
    for i in 0..100u32 {
        vec_a.push(i);
    }

    // Write without flush - just mmap, no fsync
    vec_a.write()?;

    // Now create vec_b that will read from vec_a
    // This simulates the pattern: compute vec_b derived from vec_a
    let vec_a_reader: V = V::forced_import(&database, "vec_a", version)?;

    // vec_b should see all 100 values written by vec_a
    assert_eq!(
        vec_a_reader.len(),
        100,
        "vec_b should see vec_a's written data"
    );

    for i in 0..100usize {
        assert_eq!(
            vec_a_reader.collect_one(i),
            Some(i as u32),
            "vec_b should read correct value at index {}",
            i
        );
    }

    Ok(())
}

/// Tests the full compute chain pattern:
/// 1. Compute and write vec_a
/// 2. Compute vec_b derived from vec_a, write it
/// 3. Compute vec_c derived from vec_b, write it
/// 4. Final flush for durability
fn run_compute_chain_test<V>() -> Result<(), Box<dyn std::error::Error>>
where
    V: StoredVec<I = usize, T = u32>,
{
    let version = Version::ZERO;
    let (database, _temp) = setup_test_db()?;

    // Step 1: Compute vec_a (source data)
    let mut vec_a: V = V::forced_import(&database, "chain_a", version)?;
    for i in 0..50u32 {
        vec_a.push(i * 2); // Even numbers: 0, 2, 4, ...
    }
    vec_a.write()?; // No fsync, just mmap write

    // Step 2: Compute vec_b derived from vec_a (sum of consecutive pairs)
    let vec_a_for_read: V = V::forced_import(&database, "chain_a", version)?;
    let mut vec_b: V = V::forced_import(&database, "chain_b", version)?;

    for i in 0..25usize {
        let a1 = vec_a_for_read.collect_one(i * 2).unwrap();
        let a2 = vec_a_for_read.collect_one(i * 2 + 1).unwrap();
        vec_b.push(a1 + a2);
    }
    vec_b.write()?; // No fsync, just mmap write

    // Step 3: Compute vec_c derived from vec_b (cumulative sum)
    let vec_b_for_read: V = V::forced_import(&database, "chain_b", version)?;
    let mut vec_c: V = V::forced_import(&database, "chain_c", version)?;

    let mut cumsum = 0u32;
    for val in vec_b_for_read.collect() {
        cumsum += val;
        vec_c.push(cumsum);
    }
    vec_c.write()?; // No fsync, just mmap write

    // Verify the chain computed correctly
    // vec_a: [0, 2, 4, 6, 8, 10, ...]
    // vec_b: [0+2, 4+6, 8+10, ...] = [2, 10, 18, ...]
    // vec_c: cumsum of vec_b

    let vec_c_verify: V = V::forced_import(&database, "chain_c", version)?;
    assert_eq!(vec_c_verify.len(), 25);

    let expected_b: Vec<u32> = (0..25).map(|i| (i * 4) + (i * 4 + 2)).collect();
    let expected_c: Vec<u32> = expected_b
        .iter()
        .scan(0u32, |acc, &x| {
            *acc += x;
            Some(*acc)
        })
        .collect();

    let actual_c: Vec<u32> = vec_c_verify.collect();
    assert_eq!(
        actual_c, expected_c,
        "compute chain should produce correct results"
    );

    Ok(())
}

/// Tests that write() returns the correct boolean:
/// - true if data was written
/// - false if nothing to write
fn run_write_returns_bool_test<V>() -> Result<(), Box<dyn std::error::Error>>
where
    V: StoredVec<I = usize, T = u32>,
{
    let version = Version::ZERO;
    let (database, _temp) = setup_test_db()?;

    let mut vec: V = V::forced_import(&database, "bool_test", version)?;

    // First write with no data should return false
    assert!(!vec.write()?, "write() with no data should return false");

    // Push data
    vec.push(42);

    // Write with data should return true
    assert!(vec.write()?, "write() with data should return true");

    // Second write with no new data should return false
    assert!(
        !vec.write()?,
        "write() after already written should return false"
    );

    // Push more data
    vec.push(43);

    // Write should return true again
    assert!(vec.write()?, "write() with new data should return true");

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
    fn test_write_visibility() -> Result<(), Box<dyn std::error::Error>> {
        run_write_visibility_test::<V>()
    }

    #[test]
    fn test_compute_chain() -> Result<(), Box<dyn std::error::Error>> {
        run_compute_chain_test::<V>()
    }

    #[test]
    fn test_write_returns_bool() -> Result<(), Box<dyn std::error::Error>> {
        run_write_returns_bool_test::<V>()
    }
}

#[cfg(feature = "pco")]
mod pco {
    use super::*;
    use vecdb::PcoVec;
    type V = PcoVec<usize, u32>;

    #[test]
    fn test_write_visibility() -> Result<(), Box<dyn std::error::Error>> {
        run_write_visibility_test::<V>()
    }

    #[test]
    fn test_compute_chain() -> Result<(), Box<dyn std::error::Error>> {
        run_compute_chain_test::<V>()
    }

    #[test]
    fn test_write_returns_bool() -> Result<(), Box<dyn std::error::Error>> {
        run_write_returns_bool_test::<V>()
    }
}

#[cfg(feature = "lz4")]
mod lz4 {
    use super::*;
    use vecdb::LZ4Vec;
    type V = LZ4Vec<usize, u32>;

    #[test]
    fn test_write_visibility() -> Result<(), Box<dyn std::error::Error>> {
        run_write_visibility_test::<V>()
    }

    #[test]
    fn test_compute_chain() -> Result<(), Box<dyn std::error::Error>> {
        run_compute_chain_test::<V>()
    }

    #[test]
    fn test_write_returns_bool() -> Result<(), Box<dyn std::error::Error>> {
        run_write_returns_bool_test::<V>()
    }
}

#[cfg(feature = "zstd")]
mod zstd {
    use super::*;
    use vecdb::ZstdVec;
    type V = ZstdVec<usize, u32>;

    #[test]
    fn test_write_visibility() -> Result<(), Box<dyn std::error::Error>> {
        run_write_visibility_test::<V>()
    }

    #[test]
    fn test_compute_chain() -> Result<(), Box<dyn std::error::Error>> {
        run_compute_chain_test::<V>()
    }

    #[test]
    fn test_write_returns_bool() -> Result<(), Box<dyn std::error::Error>> {
        run_write_returns_bool_test::<V>()
    }
}

#[cfg(feature = "zerocopy")]
mod zerocopy {
    use super::*;
    use vecdb::ZeroCopyVec;
    type V = ZeroCopyVec<usize, u32>;

    #[test]
    fn test_write_visibility() -> Result<(), Box<dyn std::error::Error>> {
        run_write_visibility_test::<V>()
    }

    #[test]
    fn test_compute_chain() -> Result<(), Box<dyn std::error::Error>> {
        run_compute_chain_test::<V>()
    }

    #[test]
    fn test_write_returns_bool() -> Result<(), Box<dyn std::error::Error>> {
        run_write_returns_bool_test::<V>()
    }
}
