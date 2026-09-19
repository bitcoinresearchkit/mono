use std::ops::Range;

use tempfile::TempDir;

use crate::{Database, Result, dirty_ranges::DirtyRanges};

#[test]
fn merged_ranges_match_a_byte_coverage_model() {
    let mut seed = 17u64;
    for _ in 0..64 {
        let mut ranges = DirtyRanges::default();
        let mut covered = [false; 512];
        for _ in 0..64 {
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            let start = seed as usize % covered.len();
            let end = (start + (seed >> 32) as usize % 17).min(covered.len());
            ranges.insert(start..end);
            covered[start..end].fill(true);

            let mut actual = [false; 512];
            let mut previous: Option<&Range<usize>> = None;
            for range in ranges.iter() {
                assert!(!range.is_empty());
                assert!(previous.is_none_or(|previous| previous.end < range.start));
                actual[range.clone()].fill(true);
                previous = Some(range);
            }
            assert_eq!(actual, covered);
        }
    }
}

#[test]
fn merging_many_ranges_preserves_the_untouched_suffix() {
    let mut ranges = DirtyRanges::default();
    ranges.extend((0..4096).map(|index| index * 4..index * 4 + 1));
    ranges.insert(2..12_000);
    let actual: Vec<_> = ranges.iter().cloned().collect();
    assert_eq!(&actual[..3], &[0..1, 2..12_001, 12_004..12_005]);
    assert_eq!(actual.last(), Some(&(16_380..16_381)));
}

#[test]
fn flushes_relocated_regions_once() -> Result<()> {
    let dir = TempDir::new()?;
    let db = Database::open(dir.path())?;
    let first = db.create_region_if_needed("first")?;
    let second = db.create_region_if_needed("second")?;
    first.write(b"first")?;
    second.write(b"second")?;
    // Relocation makes allocation order differ from metadata-slot order.
    first.reserve_capacity(8192)?;
    assert_eq!(db.flush()?, 2);
    assert_eq!(db.flush()?, 0);
    drop(first);
    drop(second);
    drop(db);
    let db = Database::open(dir.path())?;
    assert_eq!(
        db.get_region("first").unwrap().create_reader().read_all(),
        b"first"
    );
    assert_eq!(
        db.get_region("second").unwrap().create_reader().read_all(),
        b"second"
    );
    Ok(())
}
