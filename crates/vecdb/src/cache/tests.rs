use std::{
    ops::Range,
    sync::{
        Arc, Barrier,
        atomic::{AtomicUsize, Ordering},
        mpsc,
    },
    thread,
};

use super::{Account, Cache, CacheBudget, Request, Table, Value};

fn load(ranges: &[Range<usize>]) -> Vec<(usize, Vec<u64>)> {
    ranges
        .iter()
        .map(|range| (range.start, range.clone().map(|i| i as u64).collect()))
        .collect()
}

fn test_budget() -> &'static CacheBudget {
    Box::leak(Box::new(CacheBudget::new(1024 * 1024)))
}

#[derive(Debug)]
struct CountedValue {
    index: usize,
    clones: Arc<AtomicUsize>,
}

impl Clone for CountedValue {
    fn clone(&self) -> Self {
        self.clones.fetch_add(1, Ordering::Relaxed);
        Self {
            index: self.index,
            clones: self.clones.clone(),
        }
    }
}

#[test]
fn decoder_overread_retains_only_requested_points_and_ranges() {
    let budget = test_budget();
    let cache = Cache::new(budget);
    let clones = Arc::new(AtomicUsize::new(0));
    let indices = [11, 11, 14, 15, 19];
    cache.read_scope(|| {
        let mut out = Vec::new();
        cache.read(Request::Sorted(&indices), &mut out, |ranges| {
            assert_eq!(ranges, &[11..12, 14..16, 19..20]);
            [10..17, 18..21]
                .into_iter()
                .map(|range| {
                    let start = range.start;
                    let values = range
                        .map(|index| CountedValue {
                            index,
                            clones: clones.clone(),
                        })
                        .collect();
                    (start, values)
                })
                .collect()
        });
        assert_eq!(
            out.iter().map(|value| value.index).collect::<Vec<_>>(),
            indices
        );
        // Five output values (including the duplicate) and four retained values.
        assert_eq!(clones.load(Ordering::Relaxed), 9);
        assert_eq!(Arc::strong_count(&clones), 10);
        {
            let table = cache.table.read();
            assert_eq!(table.len(), 3);
            assert!(matches!(&table[0], (11, Value::One(_))));
            assert!(matches!(&table[1], (14, Value::Many(_, 2))));
            assert!(matches!(&table[2], (19, Value::One(_))));
        }
        for index in [10, 12, 13, 16, 17, 18, 20] {
            assert!(!cache.try_read_range(index, index + 1, || 21, &mut Vec::new()));
        }
        out.clear();
        cache.read(Request::Sorted(&indices), &mut out, |_| {
            panic!("warm read decoded")
        });
        assert_eq!(
            out.iter().map(|value| value.index).collect::<Vec<_>>(),
            indices
        );
    });
    drop(cache);
    assert_eq!(Arc::strong_count(&clones), 1);
    assert_eq!(budget.used(), 0);
}

#[test]
fn batch_merges_move_non_copy_values_without_cloning_or_losing_ownership() {
    for (old, incoming) in [
        (vec![], vec![1, 3, 5]),
        (vec![0, 4, 8], vec![2, 6]),
        (vec![4, 8], vec![0, 1, 2]),
        (vec![0, 2, 4], vec![5, 6]),
        (vec![1, 3, 5, 7], vec![0, 2, 4, 6, 8]),
    ] {
        let budget = test_budget();
        let account = Account::new(budget);
        let clones = Arc::new(AtomicUsize::new(0));
        let mut expected: Vec<_> = old.iter().chain(&incoming).copied().collect();
        expected.sort_unstable();
        let entries = |indices: Vec<usize>| {
            indices
                .into_iter()
                .map(|index| {
                    (
                        index,
                        Value::One(CountedValue {
                            index,
                            clones: clones.clone(),
                        }),
                    )
                })
                .collect()
        };
        let mut table = Table::new();
        table.insert(entries(old), &account);
        table.insert(entries(incoming), &account);
        assert_eq!(
            table
                .iter()
                .map(|(_, value)| value.slice()[0].index)
                .collect::<Vec<_>>(),
            expected
        );
        assert_eq!(clones.load(Ordering::Relaxed), 0);
        assert_eq!(Arc::strong_count(&clones), expected.len() + 1);
        drop(table);
        assert_eq!(Arc::strong_count(&clones), 1);
        assert_eq!(budget.used(), 0);
    }
}

#[test]
fn generic_fills_and_source_rewrites_match_a_model() {
    let cache = Cache::<[u64; 3]>::new(test_budget());
    let mut model: Vec<_> = (0..1024).map(|i| [0, i, 0]).collect();
    let mut seed = 17_u64;
    for turn in 0..1500 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let from = seed as usize % model.len();
        let to = (from + 1 + (seed >> 32) as usize % 180).min(model.len());
        if turn % 17 == 0 {
            cache
                .update(from, || {
                    for (at, value) in model.iter_mut().enumerate().take(to).skip(from) {
                        *value = [turn, at as u64, seed];
                    }
                    Ok(())
                })
                .unwrap();
        }
        let mut out = Vec::new();
        cache.read_scope(|| {
            cache.read(Request::Range(from, to), &mut out, |ranges| {
                ranges
                    .iter()
                    .map(|range| (range.start, model[range.clone()].to_vec()))
                    .collect()
            })
        });
        assert_eq!(out, model[from..to], "turn {turn}");
        let table = cache.table.read();
        let mut previous_end = 0;
        for (at, value) in table.iter() {
            assert!(*at >= previous_end);
            previous_end = at + value.len();
            assert_eq!(value.slice(), &model[*at..previous_end], "turn {turn}");
        }
    }
}

#[test]
fn active_reads_exclude_publication_and_callbacks_can_reenter() {
    let cache = Cache::new(test_budget());
    cache.read_scope(|| cache.read(Request::Range(0, 16), &mut Vec::new(), load));
    let barrier = Arc::new(Barrier::new(2));
    let (started_tx, started_rx) = mpsc::channel();
    let (done_tx, done_rx) = mpsc::channel();
    thread::scope(|scope| {
        let writer = cache.clone();
        let start = barrier.clone();
        scope.spawn(move || {
            start.wait();
            started_tx.send(()).unwrap();
            writer
                .update(0, || {
                    done_tx.send(()).unwrap();
                    Ok(())
                })
                .unwrap();
        });
        cache.read_scope(|| {
            barrier.wait();
            started_rx.recv().unwrap();
            cache
                .try_for_each_chunk(0, 16, load, |_, values| {
                    assert_eq!(values.len(), 16);
                    assert!(done_rx.try_recv().is_err());
                    cache.read_scope(|| {
                        cache.read(Request::Range(3, 4), &mut Vec::new(), |_| {
                            panic!("nested miss")
                        })
                    });
                    Ok::<_, ()>(())
                })
                .unwrap();
        });
        done_rx.recv().unwrap();
    });
}

#[test]
fn concurrent_misses_fill_once_and_release_every_charge() {
    let budget = Box::leak(Box::new(CacheBudget::new(128 * 1024)));
    let cache = Cache::new(budget);
    let starts = Barrier::new(8);
    let fills = AtomicUsize::new(0);
    thread::scope(|threads| {
        for _ in 0..8 {
            threads.spawn(|| {
                starts.wait();
                let mut out = Vec::new();
                cache.read_scope(|| {
                    cache.read(Request::Range(0, 4096), &mut out, |ranges| {
                        fills.fetch_add(1, Ordering::Relaxed);
                        load(ranges)
                    })
                });
                assert_eq!(out, (0..4096).collect::<Vec<_>>());
            });
        }
    });
    assert_eq!(fills.load(Ordering::Relaxed), 1);
    assert!(budget.used() > 4096 * 8);
    drop(cache);
    assert_eq!(budget.used(), 0);
}

#[test]
fn mixed_source_pressure_never_exceeds_the_shared_limit() {
    let budget = Box::leak(Box::new(CacheBudget::new(8192)));
    let caches: Vec<_> = (0..4).map(|_| Cache::new(budget)).collect();
    for turn in 0..300usize {
        let cache = &caches[turn % caches.len()];
        let from = turn * 113 % 4000;
        let count = if turn % 3 == 0 { 1 } else { 700 };
        let mut out = Vec::new();
        cache.read_scope(|| cache.read(Request::Range(from, from + count), &mut out, load));
        assert_eq!(
            out,
            (from..from + count).map(|i| i as u64).collect::<Vec<_>>()
        );
        assert!(budget.used() <= budget.limit());
    }
    drop(caches);
    assert_eq!(budget.used(), 0);
}

#[test]
fn pressure_skips_busy_sources_and_keeps_borrowed_buffers_charged() {
    let budget = test_budget();
    let cache = Cache::new(budget);
    cache.read_scope(|| cache.read(Request::Range(0, 4096), &mut Vec::new(), load));
    let table = cache.table.read();
    assert!(Account::new(budget).reserve(budget.limit()).is_none());
    assert!(!table.is_empty());
    drop(table);

    cache.read_scope(|| {
        cache
            .try_for_each_chunk(0, 4096, load, |_, values| {
                assert!(Account::new(budget).reserve(budget.limit()).is_none());
                assert!(cache.table.read().is_empty());
                assert!(budget.used() >= 4096 * size_of::<u64>());
                assert!(budget.used() <= budget.limit());
                assert_eq!(values, (0..4096).collect::<Vec<_>>());
                Ok::<_, ()>(())
            })
            .unwrap();
    });
    assert_eq!(budget.used(), 0);
    let charge = Account::new(budget).reserve(budget.limit()).unwrap();
    assert_eq!(budget.used(), budget.limit());
    drop(charge);
    assert_eq!(budget.used(), 0);
}

#[test]
fn pressure_handles_dropped_sources_and_outliving_borrows() {
    let budget = test_budget();
    let expired = Cache::new(budget);
    let live = Cache::new(budget);
    expired.read_scope(|| expired.read(Request::Range(0, 4096), &mut Vec::new(), load));
    let borrowed = expired.borrowed(Request::Range(0, 4096));
    drop(expired);

    live.read_scope(|| live.read(Request::Sorted(&[1, 3, 5]), &mut Vec::new(), load));
    assert!(Account::new(budget).reserve(budget.limit()).is_none());
    assert!(live.table.read().is_empty());
    drop(live);
    // Sweep the last dead owner, then retry with an empty registry.
    for _ in 0..2 {
        assert!(Account::new(budget).reserve(budget.limit()).is_none());
        assert!(budget.used() >= 4096 * size_of::<u64>());
        assert_eq!(borrowed[0].1.slice(), (0..4096).collect::<Vec<_>>());
    }
    drop(borrowed);
    assert_eq!(budget.used(), 0);

    let replacement = Cache::new(budget);
    replacement.read_scope(|| replacement.read(Request::Sorted(&[1, 3, 5]), &mut Vec::new(), load));
    let charge = Account::new(budget).reserve(budget.limit()).unwrap();
    assert!(replacement.table.read().is_empty());
    assert_eq!(budget.used(), budget.limit());
    drop((charge, replacement));
    assert_eq!(budget.used(), 0);
}

#[test]
fn truncation_preserves_the_allocation_and_does_not_copy_a_borrowed_prefix() {
    let budget = test_budget();
    let cache = Cache::new(budget);
    cache.read_scope(|| cache.read(Request::Range(0, 1024), &mut Vec::new(), load));
    let borrowed = cache.borrowed(Request::Range(0, 1024));
    let pointer = borrowed[0].1.slice().as_ptr();
    let charged = cache.used();
    assert_eq!(charged, budget.used());

    cache.update(1023, || Ok(())).unwrap();
    cache.extend_tail(1023, &[9999]); // Shared buffers are not copied to extend a tail.
    assert_eq!(cache.used(), charged);
    assert!(cache.try_read_range(0, 1023, || 1024, &mut Vec::new()));
    assert!(!cache.try_read_range(1023, 1024, || 1024, &mut Vec::new()));
    assert_eq!(cache.table.read()[0].1.slice().as_ptr(), pointer);
    assert_eq!(borrowed[0].1.slice()[1023], 1023);

    drop(borrowed);
    cache.extend_tail(1023, &[9999]);
    assert_eq!(cache.used(), charged);
    let mut out = Vec::new();
    assert!(cache.try_read_range(1022, 1024, || 1024, &mut out));
    assert_eq!(out, [1022, 9999]);

    let account = cache.account.clone();
    let borrowed = cache.borrowed(Request::Range(0, 1024));
    drop(cache);
    assert!(account.used() > 0);
    assert_eq!(account.used(), budget.used());
    drop(borrowed);
    assert_eq!(account.used(), 0);
    assert_eq!(budget.used(), 0);
}
