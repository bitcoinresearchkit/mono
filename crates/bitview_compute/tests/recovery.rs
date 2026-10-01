use bitview_compute::{compute_rolling_extrema_from_starts, prepare_computed};
use brk_exit::Exit;
use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, EagerVec, ImportableVec, ReadableVec, StoredVec, Version,
    WritableVec,
};

#[test]
fn group_rewind_is_visible_without_new_rows() {
    let temp = TempDir::new().unwrap();
    let db = Database::open(temp.path()).unwrap();
    let exit = Exit::new();
    let mut a = BytesVec::<usize, u64>::import(&db, "a", Version::ONE).unwrap();
    let mut b = BytesVec::<usize, u64>::import(&db, "b", Version::ONE).unwrap();
    for target in [&mut a, &mut b] {
        target
            .validate_computed_version_or_reset(Version::TWO)
            .unwrap();
        for n in 0..5 {
            target.push(n);
        }
        target.write().unwrap();
    }
    let reader = a.read_only_clone();
    b.truncate_if_needed_at(3).unwrap();
    assert_eq!(
        prepare_computed([&mut a, &mut b], Version::TWO, 5, &exit).unwrap(),
        3
    );
    assert_eq!(reader.collect(), [0, 1, 2]);
}

#[test]
fn paired_extrema_recover_partial_groups_and_shorter_sources() {
    let temp = TempDir::new().unwrap();
    let db = Database::open(temp.path()).unwrap();
    let exit = Exit::new();
    let mut values = BytesVec::<usize, u64>::import(&db, "source", Version::ONE).unwrap();
    let mut starts = BytesVec::<usize, usize>::import(&db, "starts", Version::ONE).unwrap();
    let mut min = EagerVec::<BytesVec<usize, u64>>::import(&db, "min", Version::ONE).unwrap();
    let mut max = EagerVec::<BytesVec<usize, u64>>::import(&db, "max", Version::ONE).unwrap();
    for (value, start) in [(8, 0), (2, 0), (5, 1), (9, 1), (7, 3), (1, 4)] {
        values.push(value);
        starts.push(start);
    }
    values.write().unwrap();
    starts.write().unwrap();
    for from in [0, 3, 6] {
        if from == 3 {
            max.truncate_if_needed_at(2).unwrap();
        }
        compute_rolling_extrema_from_starts(&mut min, &mut max, from, &starts, &values, &exit)
            .unwrap();
        assert_eq!(min.collect(), [8, 2, 2, 2, 7, 1]);
        assert_eq!(max.collect(), [8, 8, 5, 9, 9, 7]);
    }
    starts.truncate_if_needed_at(3).unwrap();
    starts.write().unwrap();
    compute_rolling_extrema_from_starts(&mut min, &mut max, 6, &starts, &values, &exit).unwrap();
    assert_eq!(min.read_only_clone().collect(), [8, 2, 2]);
    assert_eq!(max.read_only_clone().collect(), [8, 8, 5]);
}
