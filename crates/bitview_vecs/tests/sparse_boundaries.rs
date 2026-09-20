use std::{
    hint::black_box,
    time::{Duration, Instant},
};

use bitview_vecs::LazyAggVec;
use brk_types::{Day1, Height, StoredU64, Version};
use rangeindex::SharedRangeMap;
use tempfile::tempdir;
use vecdb::{AnySerializableVec, Database, ReadableCloneableVec, ReadableVec};

#[allow(dead_code)]
mod common;

fn median(mut read: impl FnMut(), iterations: u32) -> Duration {
    let mut samples = Vec::with_capacity(15);
    for _ in 0..15 {
        let started = Instant::now();
        for _ in 0..iterations {
            read();
        }
        samples.push(started.elapsed() / iterations);
    }
    samples.sort_unstable();
    samples[7]
}

/// Uses a shared resident map, a boxed reader, and a cached compressed
/// height source. This measures aggregation and JSON, not HTTP or cold storage.
#[test]
#[ignore = "manual sparse boundary allocation benchmark"]
fn resident_sparse_reads() {
    const HEIGHTS: usize = 966_500;
    const DAYS: usize = 6_463;
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let source = common::stored::<Height, _>(
        &db,
        "prices",
        (0..HEIGHTS).map(|i| StoredU64::from(1_000_000 + (i as u64 * 13) % 100_000)),
    );
    let starts: Vec<_> = (0..DAYS)
        .map(|i| Height::from(i * HEIGHTS / DAYS))
        .collect();
    let values = LazyAggVec::<Day1, Option<StoredU64>, Height, StoredU64>::new(
        "daily",
        Version::TWO,
        source.read_only_boxed_clone(),
        SharedRangeMap::new(starts),
    );
    let expected = values.collect();
    for (from, count) in [(0, 1), (DAYS - 1, 1), (DAYS - 365, 365), (0, DAYS)] {
        let to = from + count;
        assert_eq!(values.collect_range_at(from, to), expected[from..to]);
        let iterations = if count == DAYS { 200 } else { 2_000 };
        let collect = median(
            || {
                black_box(values.collect_range_at(black_box(from), black_box(to)));
            },
            iterations,
        );
        let json = median(
            || {
                let mut out = Vec::new();
                values.write_json(Some(from), Some(to), &mut out).unwrap();
                black_box(out);
            },
            iterations,
        );
        let point = (count == 1).then(|| {
            median(
                || {
                    black_box(values.collect_one_at(black_box(from)));
                },
                20_000,
            )
        });
        println!(
            "resident from={from} count={count}: collect={collect:?} json={json:?} point={point:?}"
        );
    }
}
