use brk_types::{Height, StoredU64, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, BytesVec, Database, EagerVec, ImportableVec, PcoVec, ReadBounds, ReadableVec,
    StoredVec, WritableVec,
};

use crate::{LazyCumulativeIndexVec, LazyIndexCountVec, LazyPreviousDeltaVec};

#[test]
fn next_boundaries_produce_cumulative_and_per_item_counts() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut first: EagerVec<PcoVec<Height, Height>> =
        EagerVec::forced_import(&db, "first", Version::ONE).unwrap();
    let mut terminal: EagerVec<PcoVec<Height, StoredU64>> =
        EagerVec::forced_import(&db, "terminal", Version::ONE).unwrap();

    for value in [0, 2, 5] {
        first.push(Height::new(value));
    }
    for value in 0_u64..6 {
        terminal.push(StoredU64::from(value));
    }
    first.write().unwrap();
    terminal.write().unwrap();

    let cumulative = LazyCumulativeIndexVec::new("cumulative", Version::ONE, &first, &terminal);
    let count = LazyIndexCountVec::new("count", Version::ONE, &first, &terminal);

    assert_eq!(
        cumulative.collect_range(Height::ZERO, Height::new(3)),
        [2_u64, 5, 6].map(StoredU64::from)
    );
    assert_eq!(
        count.collect_range(Height::ZERO, Height::new(3)),
        [2_u64, 3, 1].map(StoredU64::from)
    );
    assert_eq!(
        count.collect_range(Height::new(1), Height::new(3)),
        [3_u64, 1].map(StoredU64::from)
    );
    assert_eq!(
        cumulative.collect_one(Height::new(2)),
        Some(StoredU64::new(6))
    );
    assert_eq!(count.collect_one(Height::new(2)), Some(StoredU64::new(1)));
    assert_eq!(
        cumulative.read_sorted_at(&[0, 2, 2, 3]),
        [2_u64, 6, 6].map(StoredU64::from)
    );
    assert_eq!(
        count.read_sorted_at(&[0, 2, 2, 3]),
        [2_u64, 1, 1].map(StoredU64::from)
    );

    let mut values = Vec::new();
    cumulative.read_into_at(0, usize::MAX, &mut values);
    assert_eq!(values, [2_u64, 5, 6].map(StoredU64::from));
    assert_eq!(
        cumulative.fold_range_at(0, usize::MAX, 0, |sum, value| sum + u64::from(value)),
        13
    );
    assert_eq!(
        cumulative.try_fold_range_at(0, usize::MAX, 0, |sum, value| Ok::<_, ()>(
            sum + u64::from(value)
        )),
        Ok(13)
    );

    let mut bounds = ReadBounds::new();
    bounds.set("height", 5);
    bounds.scope(|| {
        assert_eq!(
            cumulative.collect_one(Height::new(2)),
            Some(StoredU64::new(5))
        );
        assert_eq!(count.collect_one(Height::new(2)), Some(StoredU64::new(0)));
        assert_eq!(
            count.read_sorted_at(&[0, 2, usize::MAX]),
            [2_u64, 0].map(StoredU64::from)
        );
        assert_eq!(
            cumulative.read_sorted_at(&[0, 2, usize::MAX]),
            [2_u64, 5].map(StoredU64::from)
        );
        assert_eq!(
            cumulative.fold_range_at(0, usize::MAX, 0, |sum, value| sum + u64::from(value)),
            12
        );
    });
    first.truncate_if_needed_at(1).unwrap();
    first.push(Height::new(1));
    first.push(Height::new(4));
    first.write().unwrap();
    terminal.truncate_if_needed_at(4).unwrap();
    terminal.write().unwrap();
    assert_eq!(
        count.read_sorted_at(&[0, 2, usize::MAX]),
        [1_u64, 0].map(StoredU64::from)
    );
    assert_eq!(
        cumulative.read_sorted_at(&[0, 2, usize::MAX]),
        [1_u64, 4].map(StoredU64::from)
    );
    assert_eq!(cumulative.collect(), [1_u64, 4, 4].map(StoredU64::from));
    assert!(count.read_sorted_at(&[]).is_empty());
    assert!(cumulative.read_sorted_at(&[usize::MAX]).is_empty());
}

fn check_index_chunk_boundaries<V: StoredVec<I = Height, T = Height>>() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut first = V::forced_import(&db, "first", Version::ONE).unwrap();
    let mut terminal = BytesVec::<usize, u8>::forced_import(&db, "terminal", Version::ONE).unwrap();
    let mut boundaries: Vec<usize> = (0..12_291).map(|i| i / 3).collect();
    // Keep the clamped subtraction behavior when boundaries decrease.
    boundaries[4_096] = 0;
    for &value in &boundaries {
        first.push(Height::from(value));
    }
    let terminal_len = boundaries.last().unwrap() + 7;
    for _ in 0..terminal_len {
        terminal.push(0);
    }
    first.write().unwrap();
    terminal.write().unwrap();
    let count = LazyIndexCountVec::new("count", Version::ONE, &first, &terminal);
    let cumulative = LazyCumulativeIndexVec::new("cumulative", Version::ONE, &first, &terminal);
    let delta = LazyPreviousDeltaVec::new("delta", Version::ONE, &cumulative);
    boundaries.push(terminal_len);
    let expected: Vec<_> = boundaries
        .windows(2)
        .map(|pair| StoredU64::from(pair[1].saturating_sub(pair[0]) as u64))
        .collect();
    check_index_view(&count, &expected);
    check_index_view(&delta, &expected);
    let expected: Vec<_> = boundaries[1..]
        .iter()
        .map(|&boundary| StoredU64::from(boundary as u64))
        .collect();
    check_index_view(&cumulative, &expected);
}

fn check_index_view(view: &impl ReadableVec<Height, StoredU64>, expected: &[StoredU64]) {
    for (from, to) in [
        (0, expected.len()),
        (4_095, 8_194),
        (4_096, 4_097),
        (12_290, 12_291),
        (0, 0),
        (12_291, usize::MAX),
    ] {
        let end = to.min(expected.len());
        let values = &expected[from..end];
        let mut appended = vec![StoredU64::new(99)];
        view.read_into_at(from, to, &mut appended);
        assert_eq!(&appended[1..], values);
        let sum = view.fold_range_at(from, to, 0, |sum, value| sum + u64::from(value));
        assert_eq!(sum, values.iter().copied().map(u64::from).sum::<u64>());
    }
    for index in [0, 4_095, 4_096, 8_192, 12_290, 12_291, usize::MAX] {
        assert_eq!(view.collect_one_at(index), expected.get(index).copied());
    }
    for fail_at in [1, 4_096, 4_098, expected.len()] {
        let mut visited = Vec::new();
        let result = view.try_fold_range_at(0, expected.len(), Vec::new(), |mut values, value| {
            visited.push(value);
            if visited.len() == fail_at {
                Err(fail_at)
            } else {
                values.push(value);
                Ok(values)
            }
        });
        assert_eq!(result, Err(fail_at));
        assert_eq!(visited, expected[..fail_at]);
    }
    assert_eq!(
        view.try_fold_range_at(0, usize::MAX, Vec::new(), |mut values, value| {
            values.push(value);
            Ok::<_, ()>(values)
        }),
        Ok(expected.to_vec())
    );
}

#[test]
fn raw_index_views_preserve_boundaries_across_chunks() {
    check_index_chunk_boundaries::<BytesVec<Height, Height>>();
}

#[test]
fn compressed_index_views_preserve_boundaries_between_pages() {
    check_index_chunk_boundaries::<PcoVec<Height, Height>>();
}
