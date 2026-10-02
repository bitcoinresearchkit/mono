use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use bitview_vecs::{
    LazyIndexedVec, LazyLookbackVec, LazyPreviousDeltaVec, LazyRollingRatioVec, LazyWindowVec,
};
use brk_types::{Height, StoredU64};
use tempfile::tempdir;
use vecdb::{
    AnyVec, BinaryTransform, Database, Ident, LazyVec, ReadOnlyClone, ReadableBoxedVec,
    ReadableCloneableVec, ReadableVec, ReverseOperands, Version,
};

#[allow(dead_code)]
mod common;

/// Fails immediately instead of deadlocking on a read inside a borrowed callback.
#[derive(Clone)]
struct NonReentrantSource {
    source: ReadableBoxedVec<Height, StoredU64>,
    visiting: Arc<AtomicBool>,
}

struct TestRatio;
impl BinaryTransform<StoredU64, StoredU64, StoredU64> for TestRatio {
    fn apply(a: StoredU64, b: StoredU64) -> StoredU64 {
        StoredU64::from(u64::from(a) / u64::from(b).max(1))
    }
}

#[test]
fn ratio_chunks_match_scalar_paths_and_do_not_reenter_source_reads() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let source = common::stored::<Height, _>(
        &db,
        "ratio_source",
        (0..40_000u64).map(|i| StoredU64::from((i + 1) * 3)),
    );
    let cached_source = source.read_only_clone();
    let cached = common::stored::<Height, _>(
        &db,
        "ratio_cached",
        (0..40_000u64).map(|i| StoredU64::from(i + 1)),
    );
    for (source_id, source) in [
        source.read_only_boxed_clone(),
        cached_source.read_only_boxed_clone(),
    ]
    .into_iter()
    .enumerate()
    {
        let source = NonReentrantSource {
            source,
            visiting: Arc::new(AtomicBool::new(false)),
        };
        for window in [0, 17, 20_000, 50_000] {
            let starts = common::stored::<Height, _>(
                &db,
                &format!("ratio_starts_{source_id}_{window}"),
                (0..35_000usize).map(|i| Height::from(i.saturating_sub(window))),
            );
            let ratio = LazyRollingRatioVec::<StoredU64, StoredU64, StoredU64, TestRatio>::new(
                "ratio",
                Version::ONE,
                &source,
                &cached,
                &starts,
            );
            let rolling = LazyRollingRatioVec::<
                StoredU64,
                StoredU64,
                StoredU64,
                ReverseOperands<TestRatio>,
            >::new("rolling", Version::ONE, &cached, &source, &starts);
            let cumulative = LazyIndexedVec::new(
                "cumulative",
                Version::ONE,
                &cached,
                &source,
                |_, count, numerator| TestRatio::apply(numerator, count),
            );
            check(
                ratio.read_only_boxed_clone(),
                &vec![StoredU64::from(3u64); 35_000],
            );
            check(
                rolling.read_only_boxed_clone(),
                &vec![StoredU64::from(3u64); 35_000],
            );
            check(
                cumulative.read_only_boxed_clone(),
                &vec![StoredU64::from(3u64); 40_000],
            );
        }
    }
}

impl AnyVec for NonReentrantSource {
    fn name(&self) -> &str {
        self.source.name()
    }
    fn version(&self) -> Version {
        self.source.version()
    }
    fn len(&self) -> usize {
        self.source.len()
    }
    fn index_type_to_string(&self) -> &'static str {
        self.source.index_type_to_string()
    }
    fn region_names(&self) -> Vec<String> {
        self.source.region_names()
    }
    fn value_type_to_size_of(&self) -> usize {
        self.source.value_type_to_size_of()
    }
    fn value_type_to_string(&self) -> &'static str {
        self.source.value_type_to_string()
    }
}

impl ReadableVec<Height, StoredU64> for NonReentrantSource {
    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<StoredU64>) {
        assert!(
            !self.visiting.load(Ordering::Relaxed),
            "recursive source read"
        );
        self.source.read_into_at(from, to, out);
    }
    fn for_each_chunk_at(&self, from: usize, to: usize, f: &mut dyn FnMut(usize, &[StoredU64])) {
        assert!(
            !self.visiting.swap(true, Ordering::Relaxed),
            "recursive source read"
        );
        self.source.for_each_chunk_at(from, to, f);
        self.visiting.store(false, Ordering::Relaxed);
    }
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(StoredU64)) {
        assert!(
            !self.visiting.load(Ordering::Relaxed),
            "recursive source read"
        );
        self.source.for_each_range_dyn_at(from, to, f);
    }
    fn fold_range_at<B, F: FnMut(B, StoredU64) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> B {
        assert!(
            !self.visiting.load(Ordering::Relaxed),
            "recursive source read"
        );
        self.source.fold_range_at(from, to, init, f)
    }
    fn try_fold_range_at<B, E, F: FnMut(B, StoredU64) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        assert!(
            !self.visiting.load(Ordering::Relaxed),
            "recursive source read"
        );
        self.source.try_fold_range_at(from, to, init, f)
    }
}

#[test]
fn views_do_not_recursively_read_a_source_lending_chunks() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let source = common::stored::<Height, _>(
        &db,
        "source",
        (0..40_000u64).map(|i| StoredU64::from(i * 3)),
    );
    source.collect();
    let source = NonReentrantSource {
        source: source.read_only_boxed_clone(),
        visiting: Arc::new(AtomicBool::new(false)),
    };
    let starts = common::stored::<Height, _>(
        &db,
        "starts",
        (0..40_000usize).map(|i| Height::from(i.saturating_sub(17))),
    );
    let window = LazyWindowVec::new(
        "window",
        Version::ONE,
        &source,
        &starts,
        true,
        |a: StoredU64, b: StoredU64, n| StoredU64::from(u64::from(a) + u64::from(b) + n as u64),
    );
    let lookback = LazyLookbackVec::new(
        "lookback",
        Version::ONE,
        &source,
        17,
        |a: StoredU64, b: Option<StoredU64>| {
            StoredU64::from(u64::from(a) + u64::from(b.unwrap_or_default()))
        },
    );
    let delta = LazyPreviousDeltaVec::new("delta", Version::ONE, &source);
    for view in [
        window.read_only_boxed_clone(),
        lookback.read_only_boxed_clone(),
        delta.read_only_boxed_clone(),
    ] {
        for (from, to) in [(0, 40_000), (15, 31), (16_383, 35_000)] {
            let expected = view.collect_range_at(from, to);
            let mut actual = Vec::new();
            view.for_each_chunk_at(from, to, &mut |at, values| {
                assert_eq!(at, from + actual.len());
                actual.extend_from_slice(values);
            });
            assert_eq!(actual, expected);
        }
    }
}

fn check(view: ReadableBoxedVec<Height, StoredU64>, expected: &[StoredU64]) {
    let nested = LazyVec::<Height, StoredU64, Height, StoredU64>::transformed::<Ident>(
        "nested",
        Version::ONE,
        view.clone(),
    );
    for (from, to) in [
        (0, 45_000),
        (16_380, 33_000),
        (34_990, 45_000),
        (7, 2),
        (usize::MAX, usize::MAX),
    ] {
        let expected = &expected
            [from.min(expected.len())..to.min(expected.len()).max(from.min(expected.len()))];
        assert_eq!(view.collect_range_at(from, to), expected);
        assert_eq!(nested.collect_range_at(from, to), expected);
        let mut output = vec![StoredU64::from(123u64)];
        nested.read_into_at(from, to, &mut output);
        assert_eq!(&output[1..], expected);
        output.clear();
        view.for_each_chunk_at(from, to, &mut |at, values| {
            assert!(!values.is_empty());
            assert_eq!(at, from + output.len());
            output.extend_from_slice(values);
        });
        assert_eq!(output, expected);
        output.clear();
        view.for_each_range_dyn_at(from, to, &mut |v| output.push(v));
        assert_eq!(output, expected);
        assert_eq!(
            view.fold_range_at(from, to, Vec::new(), |mut out, value| {
                out.push(value);
                out
            }),
            expected
        );
        assert_eq!(
            view.try_fold_range_at(from, to, Vec::new(), |mut out, value| {
                out.push(value);
                Ok::<_, ()>(out)
            })
            .unwrap(),
            expected
        );
    }
    let indices = [
        0,
        1,
        16_383,
        16_384,
        16_384,
        34_999,
        39_999,
        40_000,
        usize::MAX,
    ];
    let expected: Vec<_> = indices
        .iter()
        .filter_map(|&i| expected.get(i).copied())
        .collect();
    assert_eq!(view.read_sorted_at(&indices), expected);
}

#[test]
fn views_match_scalar_results_across_cached_and_fragmented_inputs() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let source = common::stored::<Height, _>(
        &db,
        "source",
        (0..40_000u64).map(|i| StoredU64::from((i + 1) * 3)),
    );
    let starts = common::stored::<Height, _>(
        &db,
        "starts",
        (0..35_000usize).map(|i| Height::from(i.saturating_sub(2016))),
    );
    let cached = source.read_only_clone();
    cached.collect();
    for source in [
        source.read_only_boxed_clone(),
        cached.read_only_boxed_clone(),
    ] {
        for inclusive in [false, true] {
            let factor = Arc::new(7u64);
            let window = LazyWindowVec::new(
                "window",
                Version::ONE,
                &source,
                &starts,
                inclusive,
                move |a: StoredU64, b: StoredU64, count| {
                    StoredU64::from((u64::from(a) - u64::from(b)) * *factor + count as u64)
                },
            );
            let expected: Vec<_> = (0..35_000usize)
                .map(|i| StoredU64::from((i.min(2016) + usize::from(inclusive)) as u64 * 22))
                .collect();
            check(window.read_only_boxed_clone(), &expected);
        }
        for lookback in [0, 1, 2016, 50_000] {
            let view = LazyLookbackVec::new(
                "lookback",
                Version::ONE,
                &source,
                lookback,
                |a: StoredU64, b: Option<StoredU64>| {
                    StoredU64::from(u64::from(a) - b.map(u64::from).unwrap_or(0))
                },
            );
            let expected: Vec<_> = (0..40_000usize)
                .map(|i| StoredU64::from((i + 1).min(lookback) as u64 * 3))
                .collect();
            check(view.read_only_boxed_clone(), &expected);
        }
        check(
            LazyPreviousDeltaVec::new("delta", Version::ONE, &source).read_only_boxed_clone(),
            &vec![StoredU64::from(3u64); 40_000],
        );
    }
}
