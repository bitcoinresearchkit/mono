#![cfg(feature = "pco")]

use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, Database, ImportableVec, PcoVec, ReadableVec, Version, WritableVec,
};

const VALUES_PER_PAGE: usize = 8 * 1024 / size_of::<u64>();

#[test]
fn constant_pages_roundtrip() -> vecdb::Result<()> {
    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let expected = vec![0; VALUES_PER_PAGE * 2];

    let mut vec = PcoVec::<usize, u64>::forced_import(&db, "constant", Version::ONE)?;
    for &value in &expected {
        vec.push(value);
    }
    vec.write()?;
    assert_eq!(vec.collect(), expected);
    drop(vec);

    let vec = PcoVec::<usize, u64>::import(&db, "constant", Version::ONE)?;
    assert_eq!(vec.collect(), expected);
    Ok(())
}

#[test]
fn shared_chunks_stop_at_metadata_blocks_and_rebuild_from_chunk_boundaries() -> vecdb::Result<()> {
    let temp = tempdir()?;
    let db = Database::open(temp.path())?;
    let mut vec = PcoVec::<usize, u64>::forced_import(&db, "values", Version::ONE)?;
    let initial: Vec<_> = (0..VALUES_PER_PAGE * 20 + 137)
        .map(|index| (index as u64).wrapping_mul(6364136223846793005))
        .collect();
    for &value in &initial {
        vec.push(value);
    }
    vec.write()?;

    // 21 pages occupy one full 136-byte metadata block and five records in
    // the second block: 136 + 8-byte base + 5 * 8-byte records.
    let pages = db.get_region(&vec.region_names()[1]).expect("pages region");
    assert_eq!(pages.meta().len(), 184);
    assert_eq!(
        vec.collect_range_at(VALUES_PER_PAGE * 15 - 7, VALUES_PER_PAGE * 16 + 7),
        initial[VALUES_PER_PAGE * 15 - 7..VALUES_PER_PAGE * 16 + 7]
    );

    let truncate_at = VALUES_PER_PAGE * 9 + 31;
    vec.truncate_if_needed_at(truncate_at)?;
    let appended: Vec<_> = (0..VALUES_PER_PAGE * 3 + 71)
        .map(|index| (index as u64).wrapping_mul(1442695040888963407))
        .collect();
    for &value in &appended {
        vec.push(value);
    }
    vec.write()?;

    let expected = initial[..truncate_at]
        .iter()
        .chain(&appended)
        .copied()
        .collect::<Vec<_>>();
    assert_eq!(vec.collect(), expected);
    drop(vec);

    let vec = PcoVec::<usize, u64>::import(&db, "values", Version::ONE)?;
    assert_eq!(vec.collect(), expected);
    Ok(())
}
