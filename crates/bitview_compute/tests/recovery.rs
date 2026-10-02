use bitview_compute::{compute_rolling_extrema_from_starts, prepare_computed};
use brk_exit::Exit;
use tempfile::TempDir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, EagerVec, ImportableVec, PcoVec, ReadableVec, StoredVec,
    Version, WritableVec,
};

#[test]
fn grouped_outputs_publish_the_shortest_valid_prefix_without_new_rows() {
    let temp = TempDir::new().unwrap();
    let db = Database::open(temp.path()).unwrap();
    let exit = Exit::new();
    let mut left = BytesVec::<usize, u64>::import(&db, "left", Version::ONE).unwrap();
    let mut right = EagerVec::<PcoVec<usize, u32>>::import(&db, "right", Version::ONE).unwrap();
    let left_reader = left.read_only_clone();
    let right_reader = right.read_only_clone();
    for (left_len, right_len, right_version, max_from, expected) in [
        (5, 3, Version::ONE, 9, 3),
        (3, 5, Version::ONE, 9, 3),
        (5, 5, Version::ONE, 2, 2),
        (5, 5, Version::TWO, 9, 0),
        (0, 5, Version::ONE, 9, 0),
        (5, 5, Version::ONE, 0, 0),
        (5, 5, Version::ONE, 5, 5),
    ] {
        left.validate_computed_version_or_reset(Version::ONE)
            .unwrap();
        right
            .validate_computed_version_or_reset(right_version)
            .unwrap();
        left.truncate_if_needed_at(0).unwrap();
        right.truncate_if_needed_at(0).unwrap();
        for n in 0..left_len {
            left.push(n as u64);
        }
        for n in 0..right_len {
            right.push(n as u32);
        }
        left.write().unwrap();
        right.write().unwrap();
        assert_eq!(
            prepare_computed(
                [&mut left as &mut dyn AnyStoredVec, &mut right],
                Version::ONE,
                max_from,
                &exit,
            )
            .unwrap(),
            expected,
        );
        assert_eq!(
            left_reader.collect(),
            (0..expected as u64).collect::<Vec<_>>()
        );
        assert_eq!(
            right_reader.collect(),
            (0..expected as u32).collect::<Vec<_>>()
        );
    }
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
