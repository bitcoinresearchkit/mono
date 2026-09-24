use std::ops::Range;

use tempfile::TempDir;

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
    assert!(region.flush()?);
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
fn merged_region_pages_keep_independent_dirty_state() -> Result<()> {
    let (db, temp) = setup_test_db()?;
    let first = db.create_region_if_needed("first")?;
    let second = db.create_region_if_needed("second")?;
    let third = db.create_region_if_needed("third")?;
    for region in [&first, &second, &third] {
        region.write(&[0])?;
    }
    db.flush()?;
    first.write_at(&[1], 0)?;
    third.write_at(&[3], 0)?;
    assert!(!second.flush()?);
    assert_eq!(
        flush_ranges(&db, &[&first, &second, &third]),
        [0..PAGE_SIZE, 2 * PAGE_SIZE..3 * PAGE_SIZE]
    );
    second.write_at(&[2], 0)?;
    let ranges = flush_ranges(&db, &[&first, &second, &third]);
    assert_eq!(ranges.len(), 1);
    assert_eq!(ranges[0], 0..3 * PAGE_SIZE);

    assert!(first.flush()?);
    assert!(flush_ranges(&db, &[&first]).is_empty());
    assert!(!first.flush()?);
    assert_eq!(
        db.flush()?,
        2,
        "flushing one region must not clear its neighbors"
    );
    assert_eq!(db.flush()?, 0);
    drop(first);
    drop(second);
    drop(third);
    drop(db);

    let db = Database::open(temp.path())?;
    for (id, value) in [("first", 1), ("second", 2), ("third", 3)] {
        assert_eq!(
            db.get_region(id).unwrap().create_reader().read_all(),
            &[value]
        );
    }
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
