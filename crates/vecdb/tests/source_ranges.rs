#[cfg(feature = "pco")]
use std::{sync::mpsc, thread, time::Duration};
#[cfg(feature = "lz4")]
use vecdb::LZ4Vec;
#[cfg(feature = "pco")]
use vecdb::PcoVec;
#[cfg(feature = "zerocopy")]
use vecdb::ZeroCopyVec;
#[cfg(feature = "zstd")]
use vecdb::ZstdVec;

use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, Budgeted, BytesVec, Database, ImportableVec, ReadableVec, StoredVec,
    Version, WritableVec,
};

fn check_source<V: StoredVec<I = usize, T = u64>>() {
    const LEN: usize = 12_345;
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let _serial = cache::TEST_LOCK.lock().unwrap();
    let budget = init_cache();
    let mut source = V::import(&db, "source", Version::ONE).unwrap();
    let captured = source.read_only_clone();
    assert!(captured.is_empty());
    for i in 0..LEN {
        source.push(i as u64);
    }
    assert!(captured.is_empty());
    source.write().unwrap();
    assert_eq!(captured.collect_one_at(42), Some(42));
    assert!(!source.read_cached_into_at(0, LEN, &mut Vec::new()));
    let indices = [42, 42, 1023, 1024, 8191, 8192, LEN - 1, LEN];
    assert_eq!(
        captured.read_sorted_at(&indices),
        indices[..7].iter().map(|&i| i as u64).collect::<Vec<_>>()
    );
    let expected: Vec<_> = (0..LEN as u64).collect();
    assert_eq!(source.collect(), expected);
    assert!(captured.read_cached_into_at(0, LEN, &mut Vec::new()));
    budget.clear();
    assert!(!captured.read_cached_into_at(0, LEN, &mut Vec::new()));
    assert_eq!(captured.collect(), expected);

    source.push(LEN as u64);
    assert_eq!(captured.len(), LEN);
    assert_eq!(captured.collect_one_at(LEN), None);
    source.write().unwrap();
    assert!(captured.read_cached_into_at(0, LEN, &mut Vec::new()));
    assert!(captured.read_cached_into_at(LEN, LEN + 1, &mut Vec::new()));
    assert_eq!(captured.collect_one_at(LEN), Some(LEN as u64));

    for from in [8193, 1023, 0] {
        source.truncate_if_needed_at(from).unwrap();
        if from != 0 {
            assert!(captured.read_cached_into_at(0, from, &mut Vec::new()));
        }
        for i in from..=LEN {
            source.push((i + from + 1) as u64);
        }
        source.write().unwrap();
        let expected: Vec<_> = (0..=LEN)
            .map(|i| {
                if i < from {
                    i as u64
                } else {
                    (i + from + 1) as u64
                }
            })
            .collect();
        assert_eq!(captured.collect(), expected);
        assert_eq!(
            source.fold_range_at(0, LEN + 1, 0u64, |sum, value| sum + value),
            expected.iter().sum::<u64>()
        );
    }
    source.reset().unwrap();
    assert!(captured.collect().is_empty());
    source.push(99);
    source.write().unwrap();
    assert_eq!(captured.collect(), [99]);
    drop(captured);
    drop(source);
    assert_eq!(budget.used(), 0);
}

#[test]
fn raw_source_ranges() {
    check_source::<BytesVec<usize, u64, Budgeted>>();
}

#[cfg(feature = "pco")]
#[test]
fn pco_source_ranges() {
    check_source::<PcoVec<usize, u64, Budgeted>>();
}

#[cfg(feature = "lz4")]
#[test]
fn lz4_source_ranges() {
    check_source::<LZ4Vec<usize, u64, Budgeted>>();
}

#[cfg(feature = "zstd")]
#[test]
fn zstd_source_ranges() {
    check_source::<ZstdVec<usize, u64, Budgeted>>();
}

#[cfg(feature = "zerocopy")]
#[test]
fn zerocopy_source_ranges() {
    check_source::<ZeroCopyVec<usize, u64, Budgeted>>();
}

#[cfg(feature = "pco")]
#[test]
fn stored_fold_keeps_its_generation_until_the_callback_finishes() {
    let _serial = cache::TEST_LOCK.lock().unwrap();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let budget = init_cache();
    let mut source = PcoVec::<usize, u64, Budgeted>::import(&db, "source", Version::ONE).unwrap();
    for i in 0..10_000 {
        source.push(i);
    }
    source.write().unwrap();
    let reader = source.read_only_clone();
    assert!(
        !reader.read_cached_into_at(0, 1, &mut Vec::new()),
        "writes must not warm a cold source"
    );
    reader.collect();
    let (entered_tx, entered_rx) = mpsc::channel();
    let (release_tx, release_rx) = mpsc::channel();
    let (started_tx, started_rx) = mpsc::channel();
    let (written_tx, written_rx) = mpsc::channel();
    thread::scope(|scope| {
        let reader = reader.clone();
        scope.spawn(move || {
            let mut paused = false;
            let mut sum = 0;
            reader.for_each_chunk_at(0, 10_000, &mut |_, values| {
                if !paused {
                    paused = true;
                    entered_tx.send(()).unwrap();
                    release_rx.recv().unwrap();
                    // Recursive reads remain safe while a writer waits.
                    assert_eq!(reader.collect_one_at(42), Some(42));
                }
                sum += values.iter().sum::<u64>();
            });
            assert_eq!(sum, (0..10_000u64).sum::<u64>());
        });
        entered_rx.recv().unwrap();
        scope.spawn(move || {
            started_tx.send(()).unwrap();
            source.truncate_if_needed_at(5_000).unwrap();
            for _ in 5_000..10_000 {
                source.push(99);
            }
            source.write().unwrap();
            written_tx.send(()).unwrap();
        });
        started_rx.recv().unwrap();
        assert!(written_rx.recv_timeout(Duration::from_millis(25)).is_err());
        budget.clear();
        assert!(
            budget.used() > 0,
            "active fold buffers remain charged after eviction"
        );
        release_tx.send(()).unwrap();
        written_rx.recv_timeout(Duration::from_secs(5)).unwrap();
    });
    assert_eq!(reader.collect_range_at(4_999, 5_002), [4_999, 99, 99]);
}

#[allow(dead_code)]
#[path = "common/cache.rs"]
mod cache;
use cache::init_cache;
