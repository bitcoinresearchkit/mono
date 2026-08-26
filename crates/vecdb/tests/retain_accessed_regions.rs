#![cfg(feature = "pco")]

use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, ImportableVec, MutableVec, PcoVec, ReadableVec,
    Version, WritableVec,
};

#[test]
fn retains_compressed_pages_and_mutable_holes() -> vecdb::Result<()> {
    let directory = tempdir()?;
    {
        let database = Database::open(directory.path())?;
        let _ = database.create_region_if_needed("stale")?;

        let mut compressed =
            PcoVec::<usize, u64>::forced_import(&database, "compressed", Version::ONE)?;
        compressed.push(10);
        compressed.push(20);
        compressed.write()?;

        let mut mutable =
            MutableVec::<BytesVec<usize, u64>>::forced_import(&database, "mutable", Version::ONE)?;
        mutable.push(1);
        mutable.push(2);
        mutable.push(3);
        mutable.write()?;
        mutable.delete(1);
        mutable.write()?;
    }

    let database = Database::open(directory.path())?;
    let compressed = PcoVec::<usize, u64>::import(&database, "compressed", Version::ONE)?;
    let mutable = MutableVec::<BytesVec<usize, u64>>::import(&database, "mutable", Version::ONE)?;
    let expected_regions = compressed
        .region_names()
        .into_iter()
        .chain(mutable.region_names())
        .collect::<Vec<_>>();

    database.retain_accessed_regions()?;

    assert!(database.get_region("stale").is_none());
    assert!(
        expected_regions
            .iter()
            .all(|name| database.get_region(name).is_some())
    );
    assert_eq!(compressed.collect_range_at(0, 2), [10, 20]);
    assert_eq!(mutable.collect_holed(), [Some(1), None, Some(3)]);

    Ok(())
}
