use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, ImportOptions, ImportableVec, MutableVec, Result, Stamp,
    Version, WritableVec,
};

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
