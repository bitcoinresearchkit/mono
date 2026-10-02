use std::ops::Range;

use crate::{Database, PAGE_SIZE, Region, Result, dirty_ranges::DirtyRanges};

use super::setup_test_db;

fn flush_ranges(db: &Database, regions: &[&Region]) -> Vec<Range<usize>> {
    let _writes = db.inner.writes.write();
    let mut ranges = DirtyRanges::default();
    for region in regions {
        // SAFETY: the exclusive mutation barrier stabilizes bounds and dirty state.
        unsafe { region.0.append_dirty_ranges(&mut ranges) };
    }
    ranges.iter().cloned().collect()
}

#[test]
fn flush_ranges_cover_dirty_pages_without_bridging_clean_pages() -> Result<()> {
    let (db, temp) = setup_test_db()?;
    let neighbor = db.create_region_if_needed("neighbor")?;
    neighbor.write(b"preserved")?;
    let region = db.create_region_if_needed("pages")?;
    let mut expected = vec![0; 7 * PAGE_SIZE + 17];
    region.write(&expected)?;
    db.flush()?;
    let start = region.meta().start();
    assert_ne!(start, 0);

    for (index, (offset, len)) in [
        (1, 1),
        (97, 2),
        (PAGE_SIZE - 1, 2),
        (PAGE_SIZE + 48, 1),
        (3 * PAGE_SIZE, 1),
        (3 * PAGE_SIZE + 200, 1),
        (6 * PAGE_SIZE - 1, 2),
        (7 * PAGE_SIZE + 16, 1),
    ]
    .into_iter()
    .enumerate()
    {
        let bytes = vec![(index + 1) as u8; len];
        region.write_at(&bytes, offset)?;
        expected[offset..offset + len].copy_from_slice(&bytes);
    }
    assert_eq!(
        flush_ranges(&db, &[&region]),
        [
            start..start + 2 * PAGE_SIZE,
            start + 3 * PAGE_SIZE..start + 4 * PAGE_SIZE,
            start + 5 * PAGE_SIZE..start + 8 * PAGE_SIZE,
        ]
    );
    assert_eq!(db.flush()?, 1);
    assert_eq!(db.flush()?, 0);
    drop(region);
    drop(neighbor);
    drop(db);

    let db = Database::open(temp.path())?;
    assert_eq!(
        db.get_region("pages").unwrap().create_reader().read_all(),
        expected
    );
    assert_eq!(
        db.get_region("neighbor")
            .unwrap()
            .create_reader()
            .read_all(),
        b"preserved"
    );
    Ok(())
}

#[test]
fn dirty_page_rounding_uses_reserved_bounds_after_truncation() -> Result<()> {
    let (db, temp) = setup_test_db()?;
    let region = db.create_region_if_needed("truncated")?;
    region.write(&vec![1; 2 * PAGE_SIZE + 1])?;
    db.flush()?;
    region.write_at(&[7], 2 * PAGE_SIZE)?;
    region.truncate(1)?;
    let ranges = flush_ranges(&db, &[&region]);
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0], 2 * PAGE_SIZE..3 * PAGE_SIZE);
    assert!(ranges[0].end <= region.meta().start() + region.meta().reserved());
    assert_eq!(db.flush()?, 1);
    drop(region);
    drop(db);
    let db = Database::open(temp.path())?;
    assert_eq!(
        db.get_region("truncated")
            .unwrap()
            .create_reader()
            .read_all(),
        &[1]
    );
    Ok(())
}

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
