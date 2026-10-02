mod common;

use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};

use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, Budgeted, BytesVec, Database, DeltaOp, DeltaSub, EagerVec, Ident, ImportableVec,
    LazyDeltaVec, LazyVec, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, StoredVec,
    UnaryTransform, Version, WritableVec,
};

struct Double;

fn assert_boxed_folds(source: &impl ReadableVec<usize, u64>, values: &[u64]) {
    for (from, to) in [
        (17, 21),
        (4090, 8195),
        (19_000, usize::MAX),
        (8, 3),
        (usize::MAX, usize::MAX),
    ] {
        let end = to.min(values.len());
        let expected = &values[from.min(end)..end];
        assert_eq!(
            source.fold_range_at(from, to, Vec::new(), |mut out, value| {
                out.push(value);
                out
            }),
            expected
        );
        assert_eq!(
            source.try_fold_range_at(from, to, 0, |sum, value| Ok::<_, ()>(sum + value)),
            Ok(expected.iter().sum())
        );
        if expected.is_empty() {
            continue;
        }
        for stop in [0, expected.len() / 2, expected.len() - 1] {
            let mut seen = Vec::new();
            let result = source.try_fold_range_at(from, to, (), |(), value| {
                seen.push(value);
                if seen.len() == stop + 1 {
                    Err("stop")
                } else {
                    Ok(())
                }
            });
            assert_eq!(result, Err("stop"));
            assert_eq!(seen, expected[..=stop]);
        }
    }
}

static DELTA_TRANSFORMS: AtomicUsize = AtomicUsize::new(0);
static DELTA_READS: AtomicUsize = AtomicUsize::new(0);
struct CountDelta;
impl DeltaOp<u64, u64> for CountDelta {
    fn ago_index(start: usize) -> Option<usize> {
        start.checked_sub(1)
    }
    fn ago_default() -> u64 {
        0
    }
    fn combine(current: u64, previous: u64, _: usize) -> u64 {
        DELTA_TRANSFORMS.fetch_add(1, Ordering::Relaxed);
        current - previous
    }
}

#[test]
fn delta_reads_only_needed_ranges_and_preserves_fallible_order() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut source = BytesVec::<usize, u64>::import(&db, "delta", Version::ONE).unwrap();
    for i in 0..40_000u64 {
        source.push((i + 1) * 3);
    }
    source.write().unwrap();
    let reads = &DELTA_READS;
    let counted = LazyVec::<usize, u64, usize, u64>::init(
        "counted",
        Version::ONE,
        ReadableBoxedVec::new(source.read_only_clone()),
        |_, value| {
            DELTA_READS.fetch_add(1, Ordering::Relaxed);
            value
        },
    );
    for window in [0, 17, 20_000, 50_000] {
        let starts = Arc::new(
            (0..35_000usize)
                .map(|i| i.saturating_sub(window))
                .collect::<Vec<_>>(),
        );
        let metadata = common::mapping(&db, starts.iter().copied());
        let delta = LazyDeltaVec::<usize, u64, u64, DeltaSub>::new(
            "delta",
            Version::ONE,
            ReadableBoxedVec::new(counted.clone()),
            metadata,
        );
        for (from, to) in [
            (0usize, 45_000usize),
            (16_380, 33_000),
            (34_990, 45_000),
            (7, 2),
            (usize::MAX, usize::MAX),
        ] {
            let expected: Vec<_> = (from.min(35_000)..to.min(35_000).max(from.min(35_000)))
                .map(|i| (i + 1).min(window + 1) as u64 * 3)
                .collect();
            assert_eq!(delta.collect_range_at(from, to), expected);
            let mut actual = Vec::new();
            delta.for_each_chunk_at(from, to, &mut |at, values| {
                assert_eq!(at, from + actual.len());
                actual.extend_from_slice(values);
            });
            assert_eq!(actual, expected);
        }
        reads.store(0, Ordering::Relaxed);
        delta.collect_range_at(34_990, 35_000);
        let expected_reads = match window {
            0 => 11,
            17 | 20_000 => 20,
            _ => 10,
        };
        assert_eq!(reads.load(Ordering::Relaxed), expected_reads);
    }
    let starts = Arc::new(
        (0..40_000usize)
            .map(|i| i.saturating_sub(17))
            .collect::<Vec<_>>(),
    );
    let delta = LazyDeltaVec::<usize, u64, u64, CountDelta>::new(
        "fallible",
        Version::ONE,
        ReadableBoxedVec::new(counted),
        common::mapping(&db, starts.iter().copied()),
    );
    DELTA_TRANSFORMS.store(0, Ordering::Relaxed);
    let mut seen = 0;
    assert_eq!(
        delta.try_fold_range_at(100, 30_000, (), |(), _| {
            seen += 1;
            if seen == 3 { Err("stop") } else { Ok(()) }
        }),
        Err("stop")
    );
    assert_eq!(DELTA_TRANSFORMS.load(Ordering::Relaxed), 3);
}

impl UnaryTransform<u64> for Double {
    fn apply(value: u64) -> u64 {
        value * 2
    }
}

#[test]
fn chunks_borrow_warm_caches_and_preserve_budget_admission_and_rewrites() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let budget = init_cache();
    let mut source =
        EagerVec::<BytesVec<usize, u64, Budgeted>>::import(&db, "source", Version::ONE).unwrap();
    let len = 20_000;
    for value in 0..len as u64 {
        source.push(value);
    }
    source.write().unwrap();
    let cached = StoredVec::read_only_clone(&source);
    let boxed = ReadableBoxedVec::new(cached.clone());
    let cached_boxed = cached.read_only_boxed_clone();
    let mut values = Vec::new();
    boxed.for_each_chunk_at(17, 21, &mut |at, chunk| {
        assert_eq!(at, 17);
        values.extend_from_slice(chunk);
    });
    assert_eq!(values, [17, 18, 19, 20]);
    assert!(cached.read_cached_into_at(17, 21, &mut Vec::new()));
    assert!(!cached.read_cached_into_at(0, len, &mut Vec::new()));

    values.clear();
    boxed.for_each_chunk_at(0, usize::MAX, &mut |at, chunk| {
        assert_eq!(at, values.len());
        values.extend_from_slice(chunk);
    });
    assert_eq!(values, (0..len as u64).collect::<Vec<_>>());
    let snapshot = values.clone();
    assert!(cached.read_cached_into_at(0, len, &mut Vec::new()));
    assert_boxed_folds(&boxed, &snapshot);
    assert_boxed_folds(&cached_boxed, &snapshot);
    let mut covered = 17;
    boxed.for_each_chunk_at(17, len + 50, &mut |at, chunk| {
        assert_eq!(at, covered);
        assert_eq!(chunk, &snapshot[at..at + chunk.len()]);
        cached_boxed.for_each_chunk_at(at, at + chunk.len(), &mut |nested_at, nested| {
            assert_eq!(nested_at, at);
            assert_eq!(
                nested.as_ptr(),
                chunk.as_ptr(),
                "warm callbacks borrow the retained allocation"
            );
            assert_eq!(nested.len(), chunk.len());
        });
        covered += chunk.len();
    });
    assert_eq!(covered, len);
    for (from, to) in [
        (2, 2),
        (50, 20),
        (len, usize::MAX),
        (usize::MAX, usize::MAX),
    ] {
        boxed.for_each_chunk_at(from, to, &mut |_, _| panic!("empty range"));
    }

    let identity = LazyVec::<usize, u64, usize, u64>::transformed::<Ident>(
        "identity",
        Version::ONE,
        boxed.clone(),
    );
    let doubled = LazyVec::<usize, u64, usize, u64>::transformed::<Double>(
        "double",
        Version::ONE,
        boxed.clone(),
    );
    let mut appended = vec![999];
    identity.read_into_at(17, 21, &mut appended);
    doubled.read_into_at(17, 21, &mut appended);
    assert_eq!(appended, [999, 17, 18, 19, 20, 34, 36, 38, 40]);

    source.truncate_if_needed_at(len - 1).unwrap();
    source.push(123);
    source.write().unwrap();
    let mut expected = snapshot.to_vec();
    expected[len - 1] = 123;
    assert_boxed_folds(&boxed, &expected);
    assert_boxed_folds(&cached_boxed, &expected);
    assert_eq!(identity.collect_range_at(len - 1, len), [123]);
    assert_eq!(doubled.collect_range_at(len - 1, len), [246]);
    assert_eq!(
        snapshot[len - 1],
        (len - 1) as u64,
        "caller-owned results remain immutable"
    );
    // A too-small budget preserves correctness through all the same read APIs.
    let denied = budget;
    let expected: Vec<_> = expected.iter().copied().cycle().take(100_000).collect();
    let len = expected.len();
    let mut tiny = BytesVec::<usize, u64, Budgeted>::import(&db, "tiny", Version::ONE).unwrap();
    for &value in &expected {
        tiny.push(value);
    }
    tiny.write().unwrap();
    let tiny = tiny.read_only_boxed_clone();
    assert_boxed_folds(&tiny, &expected);
    assert_eq!(tiny.collect(), expected);
    assert!(!tiny.read_cached_into_at(0, len, &mut Vec::new()));
    assert!(denied.used() <= denied.limit());
}

#[allow(dead_code)]
#[path = "common/cache.rs"]
mod cache;
use cache::init_cache;
