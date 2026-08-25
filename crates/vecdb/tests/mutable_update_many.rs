use tempfile::tempdir;
use vecdb::{AnyStoredVec, BytesVec, Database, ImportableVec, MutableVec, Version, WritableVec};

#[test]
fn update_many_matches_individual_updates() -> vecdb::Result<()> {
    let directory = tempdir()?;
    let database = Database::open(directory.path())?;
    let mut vec =
        MutableVec::<BytesVec<usize, u32>>::forced_import(&database, "values", Version::ONE)?;
    for _ in 0..10 {
        vec.push(0);
    }
    vec.write()?;

    vec.delete(2);
    vec.delete(4);
    vec.update(1, 11)?;
    vec.push(100);
    vec.push(101);
    vec.update_many([(2, 20), (1, 10), (4, 40), (10, 110), (11, 111), (1, 12)])?;

    assert!(vec.holes().is_empty());
    assert_eq!(
        vec.collect_holed(),
        vec![
            Some(0),
            Some(12),
            Some(20),
            Some(0),
            Some(40),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(110),
            Some(111),
        ]
    );

    vec.write()?;
    assert_eq!(
        vec.collect_holed(),
        vec![
            Some(0),
            Some(12),
            Some(20),
            Some(0),
            Some(40),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(0),
            Some(110),
            Some(111),
        ]
    );
    Ok(())
}

#[test]
fn update_many_rejects_the_whole_out_of_range_batch() -> vecdb::Result<()> {
    let directory = tempdir()?;
    let database = Database::open(directory.path())?;
    let mut vec =
        MutableVec::<BytesVec<usize, u32>>::forced_import(&database, "values", Version::ONE)?;
    vec.push(1);

    assert!(vec.update_many([(0, 2), (1, 3)]).is_err());
    assert_eq!(vec.collect_holed(), vec![Some(1)]);
    Ok(())
}
