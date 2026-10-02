use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, ImportOptions, ImportableVec, MutableVec, Result, Stamp,
    Version, WritableVec,
};

#[test]
fn update_many_resolves_holes_pending_values_and_duplicate_indexes() -> Result<()> {
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
    let expected = [0, 12, 20, 0, 40, 0, 0, 0, 0, 0, 110, 111].map(Some);
    assert_eq!(vec.collect_holed(), expected);
    vec.write()?;
    assert_eq!(vec.collect_holed(), expected);
    Ok(())
}

#[test]
fn update_many_preserves_stamped_rollback() -> Result<()> {
    for stored_len in [0, 8, 16] {
        let directory = tempdir()?;
        let database = Database::open(directory.path())?;
        let options =
            ImportOptions::new(&database, "values", Version::ONE).with_saved_stamped_changes(4);
        let mut vec = MutableVec::<BytesVec<usize, u32>>::import_with(options)?;
        vec.fill_to(stored_len, 0)?;
        vec.stamped_write_with_changes(Stamp::new(1))?;
        vec.fill_to(16, 0)?;
        vec.update(1, 40)?;

        // Sorted replacements span both sides of the stored/appended boundary,
        // overwrite a pending mutation, and preserve last-wins duplicates.
        vec.update_many([(0, 100), (1, 101), (1, 102), (7, 103), (8, 104), (15, 105)])?;
        let mut expected = vec![Some(0); 16];
        for (index, value) in [(0, 100), (1, 102), (7, 103), (8, 104), (15, 105)] {
            expected[index] = Some(value);
        }
        assert_eq!(vec.collect_holed(), expected);
        assert!(vec.update_many([(0, 2), (16, 3)]).is_err());
        assert_eq!(vec.collect_holed(), expected);
        vec.stamped_write_with_changes(Stamp::new(2))?;
        assert_eq!(vec.collect_holed(), expected);

        vec.rollback()?;
        assert_eq!(vec.stamp(), Stamp::new(1));
        assert_eq!(vec.collect_holed(), vec![Some(0); stored_len]);
    }
    Ok(())
}

#[test]
fn staged_mutations_survive_reopen_and_fork_rollback() -> Result<()> {
    let directory = tempdir()?;
    {
        let database = Database::open(directory.path())?;
        let options =
            ImportOptions::new(&database, "values", Version::ONE).with_saved_stamped_changes(4);
        let mut vec = MutableVec::<BytesVec<usize, u32>>::import_with(options)?;
        vec.extend([10, 20]);
        vec.stamped_write_with_changes(Stamp::new(1))?;
        vec.extend([30, 40]);
        vec.pushed_mut()[0] = 31;
        vec.update(0, 11)?;
        vec.stamped_write_with_changes(Stamp::new(2))?;
        vec.flush()?;
    }
    {
        let database = Database::open(directory.path())?;
        let options =
            ImportOptions::new(&database, "values", Version::ONE).with_saved_stamped_changes(4);
        let mut vec = MutableVec::<BytesVec<usize, u32>>::import_with(options)?;
        assert_eq!(
            vec.collect_holed(),
            [Some(11), Some(20), Some(31), Some(40)]
        );
        assert_eq!(vec.rollback_before(Stamp::new(2))?, Stamp::new(1));
        assert_eq!(vec.collect_holed(), [Some(10), Some(20)]);
        vec.extend([50, 60]);
        vec.pushed_mut()[1] = 61;
        vec.stamped_write_with_changes(Stamp::new(3))?;
        vec.flush()?;
    }
    let database = Database::open(directory.path())?;
    let options =
        ImportOptions::new(&database, "values", Version::ONE).with_saved_stamped_changes(4);
    let mut vec = MutableVec::<BytesVec<usize, u32>>::import_with(options)?;
    assert_eq!(
        vec.collect_holed(),
        [Some(10), Some(20), Some(50), Some(61)]
    );
    assert_eq!(vec.rollback_before(Stamp::new(3))?, Stamp::new(1));
    assert_eq!(vec.collect_holed(), [Some(10), Some(20)]);
    Ok(())
}
