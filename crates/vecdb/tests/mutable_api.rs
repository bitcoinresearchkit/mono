use std::collections::BTreeSet;

use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, ImportOptions, ImportableVec, MutableVec,
    ReadableVec, Result, Stamp, StoredVec, Version, WritableVec,
};

#[cfg(feature = "zerocopy")]
use vecdb::ZeroCopyVec;

macro_rules! mutation_roundtrip {
    ($vec:ty) => {{
        type V = $vec;
        let directory = tempdir()?;
        {
            let db = Database::open(directory.path())?;
            let options =
                ImportOptions::new(&db, "values", Version::ONE).with_saved_stamped_changes(4);
            let mut vec = MutableVec::<V>::import_with(options)?;
            vec.reserve_pushed(6);
            assert!(vec.is_empty());
            for value in 10..14 {
                vec.push(value);
            }
            vec.stamped_write_with_changes(Stamp::new(1))?;
            let published = vec.read_only_clone();
            let reader = vec.reader();
            assert_eq!(vec.get_with_reader(2, &reader), Some(12));
            assert_eq!(vec.get_with_reader_at(4, &reader), None);
            assert!(vec.prev_holes().is_empty());
            assert!(vec.prev_updated().is_empty());

            vec.update(2, 22)?;
            vec.push(14);
            vec.update_at(4, 24)?;
            vec.delete(1);
            vec.delete_at(99);
            assert_eq!(vec.updated().get(&2), Some(&22));
            assert_eq!(vec.get_with_reader_at(2, &reader), Some(22));
            assert_eq!(vec.get_with_reader_at(4, &reader), Some(24));
            assert_eq!(vec.take(3, &reader), Some(13));
            assert_eq!(vec.take_at(0, &reader), Some(10));
            assert_eq!(vec.take_at(0, &reader), None);
            assert_eq!(vec.holes(), &BTreeSet::from([0, 1, 3]));
            assert_eq!(vec.fill_first_hole_or_push(100)?, 0);
            assert_eq!(vec.fill_first_hole_or_push(101)?, 1);
            assert_eq!(vec.holes().first(), Some(&3));
            assert_eq!(
                vec.collect_holed(),
                [Some(100), Some(101), Some(22), None, Some(24)]
            );
            assert_eq!(published.collect(), [10, 11, 12, 13]);
            drop(reader);

            vec.stamped_write_with_changes(Stamp::new(2))?;
            assert_eq!(vec.prev_holes(), &BTreeSet::from([3]));
            assert!(vec.prev_updated().is_empty());
            assert_eq!(published.collect(), [100, 101, 22, 24]);
            let reader = vec.reader();
            assert_eq!(vec.get_with_reader_at(3, &reader), None);
            assert_eq!(vec.get_with_reader_at(4, &reader), Some(24));
            drop(reader);

            vec.rollback()?;
            assert_eq!(vec.stamp(), Stamp::new(1));
            assert_eq!(
                vec.collect_holed(),
                [Some(10), Some(11), Some(12), Some(13)]
            );
            assert!(vec.holes().is_empty());
            vec.stamped_write_with_changes(Stamp::new(1))?;
            vec.flush()?;
        }
        let db = Database::open(directory.path())?;
        let mut vec = MutableVec::<V>::import(&db, "values", Version::ONE)?;
        assert_eq!(
            vec.collect_holed(),
            [Some(10), Some(11), Some(12), Some(13)]
        );
        assert!(vec.update_at(4, 999).is_err());
        assert_eq!(vec.len(), 4);
        assert_eq!(vec.fill_first_hole_or_push(14)?, 4);
        assert_eq!(vec.len(), 5);
        vec.delete_at(0);
        vec.update_at(0, 20)?;
        assert!(vec.holes().is_empty());
        vec.delete_at(1);
        vec.write()?;
        drop(vec);
        let vec = MutableVec::<V>::forced_import(&db, "values", Version::TWO)?;
        assert!(vec.is_empty());
        assert!(vec.holes().is_empty());
        assert!(
            vec.region_names()
                .iter()
                .all(|name| !name.ends_with("_holes"))
        );
        Ok(())
    }};
}

#[test]
fn bytes_mutation_api_roundtrip() -> Result<()> {
    mutation_roundtrip!(BytesVec<usize, u32>)
}

#[cfg(feature = "zerocopy")]
#[test]
fn zerocopy_mutation_api_roundtrip() -> Result<()> {
    mutation_roundtrip!(vecdb::ZeroCopyVec<usize, u32>)
}

#[cfg(feature = "zerocopy")]
#[test]
fn zerocopy_reader_merges_updates_and_holes() -> Result<()> {
    let directory = tempdir()?;
    let db = Database::open(directory.path())?;
    let mut vec = MutableVec::<ZeroCopyVec<usize, u32>>::import(&db, "values", Version::ONE)?;
    for value in 10..13 {
        vec.push(value);
    }
    vec.write()?;
    let reader = vec.reader();
    assert_eq!(vec.get_append_only(0, &reader), Some(10));
    vec.update(0, 20)?;
    vec.delete(1);
    assert_eq!(vec.get_with_reader(0, &reader), Some(20));
    assert_eq!(vec.get_with_reader(1, &reader), None);
    assert_eq!(vec.get_with_reader(2, &reader), Some(12));
    Ok(())
}

#[test]
fn read_only_clone_tracks_published_holes() -> Result<()> {
    let directory = tempdir()?;
    let database = Database::open(directory.path())?;
    let mut vec = MutableVec::<BytesVec<usize, u32>>::forced_import(
        &database,
        "read_only_holes",
        Version::ONE,
    )?;
    vec.push(1);
    vec.push(2);
    vec.push(3);
    vec.write()?;

    let read_only = vec.read_only_clone();
    vec.delete_at(1);
    vec.write()?;

    assert_eq!(read_only.collect(), vec![1, 3]);
    assert_eq!(read_only.collect_one_at(1), None);

    vec.update_at(1, 4)?;
    vec.write()?;
    assert_eq!(read_only.collect(), vec![1, 4, 3]);

    Ok(())
}
