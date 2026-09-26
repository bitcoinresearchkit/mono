use brk_exit::Exit;
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, BytesVec, Database, EagerVec, ImportableVec, ReadableVec, StoredVec,
    Version, WritableVec,
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
                .map(|j| {
                    if j % 7 == 0 {
                        u64::MAX
                    } else {
                        (i * 11 + j) as u64
                    }
                })
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

    let expected: Vec<_> = groups
        .iter()
        .map(|group| group.iter().copied().fold(0u64, u64::saturating_add))
        .collect();
    let expected_cumulative: Vec<_> = groups
        .iter()
        .scan(0, |total, group| {
            *total += group.iter().map(|value| value % 10).sum::<u64>();
            Some(*total)
        })
        .collect();
    let mut output: EagerVec<V> = EagerVec::forced_import(&db, "all", Version::ONE).unwrap();
    let mut cumulative: EagerVec<V> =
        EagerVec::forced_import(&db, "grouped_cumulative", Version::ONE).unwrap();
    for phase in 0..5 {
        if phase == 2 {
            drop(output);
            output = EagerVec::forced_import(&db, "all", Version::ONE).unwrap();
            drop(cumulative);
            cumulative = EagerVec::forced_import(&db, "grouped_cumulative", Version::ONE).unwrap();
        }
        if phase == 4 {
            output
                .validate_computed_version_or_reset(Version::ZERO)
                .unwrap();
            cumulative
                .validate_computed_version_or_reset(Version::ZERO)
                .unwrap();
        }
        let from = if phase == 3 { 7 } else { groups.len() };
        output
            .compute_sum_from_indexes(from, &first, &counts, &source, &exit)
            .unwrap();
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
        assert_eq!(output.collect(), expected, "phase={phase}");
        assert_eq!(cumulative.collect(), expected_cumulative, "phase={phase}");
    }

    let empty_first: BytesVec<usize, usize> =
        BytesVec::forced_import(&db, "empty_first", Version::ONE).unwrap();
    let mut output: EagerVec<V> = EagerVec::forced_import(&db, "missing", Version::ONE).unwrap();
    output
        .compute_sum_from_indexes(0, &empty_first, &counts, &source, &exit)
        .unwrap();
    assert!(output.collect().is_empty());
}

#[test]
fn empty_groups_preserve_the_cumulative_signed_zero() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut first = BytesVec::<usize, usize>::forced_import(&db, "first", Version::ONE).unwrap();
    let mut counts = BytesVec::<usize, usize>::forced_import(&db, "counts", Version::ONE).unwrap();
    let source = BytesVec::<usize, f64>::forced_import(&db, "source", Version::ONE).unwrap();
    for _ in 0..3 {
        first.push(0);
        counts.push(0);
    }
    let mut target =
        EagerVec::<BytesVec<usize, f64>>::forced_import(&db, "cumulative", Version::ONE).unwrap();
    target
        .validate_computed_version_or_reset(first.version() + counts.version() + source.version())
        .unwrap();
    target.push(-0.0);
    target
        .compute_cumulative_sum_from_indexes(
            1,
            &first,
            &counts,
            &source,
            |value| value,
            &Exit::new(),
        )
        .unwrap();
    assert_eq!(
        target
            .collect()
            .into_iter()
            .map(f64::to_bits)
            .collect::<Vec<_>>(),
        [(-0.0f64).to_bits(); 3]
    );
}

#[test]
fn raw_indexed_sums_preserve_groups_saturation_and_resume() {
    check_indexed_sums::<BytesVec<usize, u64>>();
}

#[cfg(feature = "pco")]
#[test]
fn compressed_indexed_sums_preserve_groups_saturation_and_resume() {
    check_indexed_sums::<PcoVec<usize, u64>>();
}

fn check_cumulative<V: StoredVec<I = usize, T = u64>>() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut source1 = V::forced_import(&db, "source1", Version::ONE).unwrap();
    let mut source2 = V::forced_import(&db, "source2", Version::ONE).unwrap();
    for i in 0..32u64 {
        source1.push(i * 3);
        source2.push(i * 5);
    }
    source1.write().unwrap();
    source2.write().unwrap();
    let exit = Exit::new();
    for mode in 0..3 {
        let name = format!("cumulative_{mode}");
        let expected: Vec<_> = (0..32u64)
            .scan(0, |total, i| {
                *total += i * [3, 8, 11][mode];
                Some(*total)
            })
            .collect();
        let mut output: EagerVec<V> = EagerVec::forced_import(&db, &name, Version::ONE).unwrap();
        for phase in 0..5 {
            if phase == 2 {
                drop(output);
                output = EagerVec::forced_import(&db, &name, Version::ONE).unwrap();
            }
            if phase == 4 {
                output
                    .validate_computed_version_or_reset(Version::ZERO)
                    .unwrap();
            }
            let from = if phase == 3 { 11 } else { 32 };
            match mode {
                0 => output.compute_cumulative(from, &source1, &exit),
                1 => output.compute_cumulative_transformed_binary(
                    from,
                    &source1,
                    &source2,
                    |a, b| a + b,
                    &exit,
                ),
                _ => output.compute_cumulative_transformed_binary(
                    from,
                    &source1,
                    &source2,
                    |a, b| a * 2 + b,
                    &exit,
                ),
            }
            .unwrap();
            assert_eq!(output.collect(), expected, "mode={mode} phase={phase}");
        }
    }
}

#[test]
fn raw_cumulative_paths_preserve_stored_totals() {
    check_cumulative::<BytesVec<usize, u64>>();
}

#[cfg(feature = "pco")]
#[test]
fn compressed_cumulative_paths_preserve_stored_totals() {
    check_cumulative::<PcoVec<usize, u64>>();
}
