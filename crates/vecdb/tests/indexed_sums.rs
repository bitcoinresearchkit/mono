use brk_exit::Exit;
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, EagerVec, ImportableVec, ReadableVec, StoredVec, Version,
    WritableVec,
};

#[cfg(feature = "pco")]
use vecdb::PcoVec;

fn check_indexed_sums<V: StoredVec<I = usize, T = u64>>() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut first: BytesVec<usize, usize> =
        BytesVec::forced_import(&db, "first", Version::ONE).unwrap();
    let mut counts: BytesVec<usize, usize> =
        BytesVec::forced_import(&db, "counts", Version::ONE).unwrap();
    let mut source: V = V::forced_import(&db, "source", Version::ONE).unwrap();
    let groups: Vec<Vec<u64>> = (0..128usize)
        .map(|i| {
            (0..[0, 0, 1, 0, 3, 17, 0, 2, 0][i % 9])
                .map(|j| (i * 11 + j) as u64)
                .collect()
        })
        .collect();
    let mut offset = 0;
    for group in &groups {
        first.push(offset);
        counts.push(group.len());
        offset += group.len();
        for &value in group {
            source.push(value);
        }
    }
    // Ignore an orphan count whose first index has not been written yet.
    counts.push(0);
    first.write().unwrap();
    counts.write().unwrap();
    source.write().unwrap();
    let exit = Exit::new();

    let expected_cumulative: Vec<_> = groups
        .iter()
        .scan(0, |total, group| {
            *total += group.iter().map(|value| value % 10).sum::<u64>();
            Some(*total)
        })
        .collect();
    let mut cumulative: EagerVec<V> =
        EagerVec::forced_import(&db, "grouped_cumulative", Version::ONE).unwrap();
    for phase in 0..5 {
        if phase == 2 {
            drop(cumulative);
            cumulative = EagerVec::forced_import(&db, "grouped_cumulative", Version::ONE).unwrap();
        }
        if phase == 4 {
            cumulative
                .validate_computed_version_or_reset(Version::ZERO)
                .unwrap();
        }
        let from = if phase == 3 { 7 } else { groups.len() };
        cumulative
            .compute_cumulative_sum_from_indexes(
                from,
                &first,
                &counts,
                &source,
                |value| value % 10,
                &exit,
            )
            .unwrap();
        assert_eq!(cumulative.collect(), expected_cumulative, "phase={phase}");
    }

    let empty_first: BytesVec<usize, usize> =
        BytesVec::forced_import(&db, "empty_first", Version::ONE).unwrap();
    let mut output: EagerVec<V> = EagerVec::forced_import(&db, "missing", Version::ONE).unwrap();
    output
        .compute_cumulative_sum_from_indexes(
            0,
            &empty_first,
            &counts,
            &source,
            |value| value,
            &exit,
        )
        .unwrap();
    assert!(output.collect().is_empty());
}

#[test]
fn raw_indexed_sums_preserve_groups_and_resume() {
    check_indexed_sums::<BytesVec<usize, u64>>();
}

#[cfg(feature = "pco")]
#[test]
fn compressed_indexed_sums_preserve_groups_and_resume() {
    check_indexed_sums::<PcoVec<usize, u64>>();
}
