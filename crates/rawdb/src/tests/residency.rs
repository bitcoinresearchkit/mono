use crate::{PAGE_SIZE, Result, region::residency::is_range_resident};

use super::setup_test_db;

#[test]
fn residency_probe_handles_empty_and_invalid_ranges() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    db.set_min_len(PAGE_SIZE)?;
    let mmap = db.inner.data.read();
    assert!(is_range_resident(&mmap, 0, 0));
    assert!(is_range_resident(&mmap, mmap.len(), 0));
    assert!(!is_range_resident(&mmap, mmap.len() + 1, 0));
    assert!(!is_range_resident(&mmap, mmap.len(), 1));
    assert!(!is_range_resident(&mmap, usize::MAX, usize::MAX));
    Ok(())
}

#[cfg(unix)]
#[test]
fn test_region_residency_hint() -> Result<()> {
    let (db, _temp) = setup_test_db()?;
    let region = db.create_region_if_needed("resident")?;

    assert!(region.prefers_mmap(0, 0));
    assert!(region.prefers_mmap(0, 1));

    let bytes = vec![42; 256 * 1024];
    region.write(&bytes)?;

    assert!(region.prefers_mmap(0, bytes.len()));
    assert!(region.prefers_mmap(bytes.len() - 1, 2));
    assert!(!region.prefers_mmap(bytes.len() - 1, 128 * 1024));
    assert!(!region.prefers_mmap(usize::MAX, 128 * 1024));

    // Cross several sampling windows and exercise an unaligned subrange.
    let more = vec![7; 32 * 1024 * 1024];
    region.write(&more)?;
    assert!(region.prefers_mmap(1, bytes.len() + more.len() - 2));

    Ok(())
}
