#[allow(dead_code)]
mod common;

use std::sync::atomic::{AtomicUsize, Ordering};

use bitview_vecs::{DailyView, LastDay, RepeatDay};
use brk_types::{Day1, Height, StoredU64};
use tempfile::tempdir;
use vecdb::{
    Database, LazyVec, ReadOnlyClone, ReadableBoxedVec, ReadableCloneableVec, ReadableVec,
    UnaryTransform, Version,
};

struct Twice;
impl UnaryTransform<Option<StoredU64>> for Twice {
    fn apply(value: Option<StoredU64>) -> Option<StoredU64> {
        value.map(|v| StoredU64::from(u64::from(v) * 2))
    }
}

fn check(view: ReadableBoxedVec<Height, Option<StoredU64>>, expected: &[Option<StoredU64>]) {
    let nested = LazyVec::<Height, Option<StoredU64>, Height, Option<StoredU64>>::transformed::<
        Twice,
    >("nested", Version::ONE, view.clone());
    for indices in [
        vec![],
        vec![0],
        vec![
            0,
            0,
            1,
            3,
            4,
            4,
            7,
            143,
            144,
            28_799,
            28_800,
            39_999,
            40_000,
            usize::MAX,
        ],
        (0..expected.len()).step_by(17).collect(),
    ] {
        let selected: Vec<_> = indices
            .iter()
            .filter_map(|&i| expected.get(i).copied())
            .collect();
        assert_eq!(view.read_sorted_at(&indices), selected);
        let mut appended = vec![None];
        view.read_sorted_into_at(&indices, &mut appended);
        assert_eq!(&appended[1..], selected);
    }
    for (from, to) in [
        (0usize, 45_000usize),
        (16_380, 33_000),
        (28_795, 45_000),
        (3, 7),
        (7, 2),
        (usize::MAX, usize::MAX),
    ] {
        let selected = &expected
            [from.min(expected.len())..to.min(expected.len()).max(from.min(expected.len()))];
        assert_eq!(view.collect_range_at(from, to), selected);
        assert_eq!(
            nested.collect_range_at(from, to),
            selected
                .iter()
                .copied()
                .map(Twice::apply)
                .collect::<Vec<_>>()
        );
        let mut actual = Vec::new();
        view.for_each_chunk_at(from, to, &mut |at, values| {
            assert_eq!(at, from + actual.len());
            assert!(!values.is_empty());
            actual.extend_from_slice(values);
        });
        assert_eq!(actual, selected);
        assert_eq!(
            view.fold_range_at(from, to, Vec::new(), |mut out, value| {
                out.push(value);
                out
            }),
            selected
        );
        assert_eq!(
            view.try_fold_range_at(from, to, Vec::new(), |mut out, value| {
                out.push(value);
                Ok::<_, ()>(out)
            })
            .unwrap(),
            selected
        );
        let mut appended = vec![None];
        view.read_into_at(from, to, &mut appended);
        assert_eq!(&appended[1..], selected);
    }
}

#[test]
fn daily_chunks_match_scalar_paths_including_missing_and_duplicate_days() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let source = common::stored::<Day1, _>(&db, "daily", (0..200u64).map(StoredU64::from));
    let repeated =
        common::stored::<Height, _>(&db, "repeat", (0..40_000usize).map(|i| Day1::from(i / 144)));
    let last = common::stored::<Height, _>(
        &db,
        "last",
        [0usize, 0, 1, 7, 13, 199, 200, 210].map(Day1::from),
    );
    let cached_source = source.read_only_clone();
    let cached_repeated = repeated.read_only_clone();
    let cached_last = last.read_only_clone();
    for (source, repeated, last) in [
        (
            source.read_only_boxed_clone(),
            repeated.read_only_boxed_clone(),
            last.read_only_boxed_clone(),
        ),
        (
            cached_source.read_only_boxed_clone(),
            cached_repeated.read_only_boxed_clone(),
            cached_last.read_only_boxed_clone(),
        ),
    ] {
        let repeat = DailyView::<Height, StoredU64, RepeatDay>::new(
            "repeat",
            Version::ONE,
            &source,
            &repeated,
        );
        check(
            repeat.read_only_boxed_clone(),
            &(0..40_000usize)
                .map(|i| (i / 144 < 200).then(|| StoredU64::from((i / 144) as u64)))
                .collect::<Vec<_>>(),
        );
        let last =
            DailyView::<Height, StoredU64, LastDay>::new("last", Version::ONE, &source, &last);
        check(
            last.read_only_boxed_clone(),
            &[
                None,
                Some(0u64),
                Some(6),
                Some(12),
                Some(198),
                Some(199),
                None,
                None,
            ]
            .map(|v| v.map(StoredU64::from)),
        );
        let mut seen = 0;
        assert_eq!(
            repeat.try_fold_range_at(0, 40_000, (), |(), _| {
                seen += 1;
                if seen == 3 { Err("stop") } else { Ok(()) }
            }),
            Err("stop")
        );
        assert_eq!(seen, 3);
    }
}

static DAILY_READS: AtomicUsize = AtomicUsize::new(0);

#[test]
fn last_day_keeps_selective_source_reads() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let source =
        common::stored::<Day1, _>(&db, "selective_source", (0..20_000u64).map(StoredU64::from));
    let source = LazyVec::<Day1, StoredU64, Day1, StoredU64>::init(
        "counted",
        Version::ONE,
        source.read_only_boxed_clone(),
        |_, value| {
            DAILY_READS.fetch_add(1, Ordering::Relaxed);
            value
        },
    );
    let mapping = common::stored::<Height, _>(
        &db,
        "selective_mapping",
        (0..200usize).map(|i| Day1::from(i * 100)),
    );
    let view =
        DailyView::<Height, StoredU64, LastDay>::new("last", Version::ONE, &source, &mapping);
    DAILY_READS.store(0, Ordering::Relaxed);
    let mut actual = Vec::new();
    view.for_each_chunk_at(180, 183, &mut |_, values| actual.extend_from_slice(values));
    assert_eq!(
        actual,
        [
            Some(StoredU64::from(18_099u64)),
            Some(StoredU64::from(18_199u64)),
            Some(StoredU64::from(18_299u64))
        ]
    );
    assert_eq!(DAILY_READS.load(Ordering::Relaxed), 3);
}
