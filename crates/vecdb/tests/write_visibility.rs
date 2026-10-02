//! Writes must be visible to fresh imports before a flush.

use tempfile::TempDir;
#[cfg(feature = "lz4")]
use vecdb::LZ4Vec;
#[cfg(feature = "pco")]
use vecdb::PcoVec;
#[cfg(feature = "zerocopy")]
use vecdb::ZeroCopyVec;
#[cfg(feature = "zstd")]
use vecdb::ZstdVec;

use vecdb::{BytesVec, Database, Result, StoredVec, Version};

fn run<V: StoredVec<I = usize, T = u32>>() -> Result<()> {
    let directory = TempDir::new()?;
    let db = Database::open(directory.path())?;
    let mut writer = V::import(&db, "values", Version::ONE)?;
    assert!(!writer.write()?);

    let expected = [2, 6, 10, 14, 18];
    for end in [3, 5] {
        for &value in &expected[writer.len()..end] {
            writer.push(value);
        }
        assert!(writer.write()?);
        let reader = V::import(&db, "values", Version::ONE)?;
        assert_eq!(reader.collect(), expected[..end]);
        assert!(!writer.write()?);
        assert_eq!(reader.collect(), expected[..end]);
    }
    Ok(())
}

#[test]
fn bytes() -> Result<()> {
    run::<BytesVec<usize, u32>>()
}

#[cfg(feature = "zerocopy")]
#[test]
fn zerocopy() -> Result<()> {
    run::<ZeroCopyVec<usize, u32>>()
}

#[cfg(feature = "pco")]
#[test]
fn pco() -> Result<()> {
    run::<PcoVec<usize, u32>>()
}

#[cfg(feature = "lz4")]
#[test]
fn lz4() -> Result<()> {
    run::<LZ4Vec<usize, u32>>()
}

#[cfg(feature = "zstd")]
#[test]
fn zstd() -> Result<()> {
    run::<ZstdVec<usize, u32>>()
}
