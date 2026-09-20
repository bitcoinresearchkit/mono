use bitview_vecs::{LazySmaVec, import_cached};
use brk_exit::Exit;
use brk_types::{Cents, Height, StoredU64, Version};
use tempfile::tempdir;
use vecdb::{AnyStoredVec, Database, ReadableCloneableVec, ReadableVec, WritableVec};

use super::compute_sma_prefix;
use crate::test_cache::init_cache;

#[test]
fn stored_sma_prefix_survives_append_rewrite_eviction_and_reopen() {
    let budget = init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut prices = import_cached::<Height, Cents>(&db, "prices", Version::ONE).unwrap();
    let mut prefix = import_cached::<Height, StoredU64>(&db, "prefix", Version::ONE).unwrap();
    let mut starts = import_cached::<Height, Height>(&db, "starts", Version::ONE).unwrap();
    for i in 0usize..4300 {
        starts.push(Height::from(i.saturating_sub(73)));
    }
    starts.write().unwrap();
    let reader = prefix.read_only_boxed_clone();
    let sma = LazySmaVec::new(
        "sma",
        Version::ONE,
        reader.clone(),
        starts.read_only_boxed_clone(),
    );
    let exit = Exit::default();
    let mut expected_prices = Vec::new();
    let mut expected_prefix = Vec::new();

    for (from, end, offset) in [
        (0, 4096, 0),
        (4096, 4300, 0),
        (1023, 4300, 500),
        (0, 4300, 17),
    ] {
        prices.truncate_if_needed_at(from).unwrap();
        expected_prices.truncate(from);
        for i in from..end {
            let price = Cents::new(100 + ((i * 37 + offset) % 1000) as u64);
            prices.push(price);
            expected_prices.push(price);
        }
        prices.write().unwrap();
        compute_sma_prefix(&mut prefix, Height::from(from), &prices, &exit).unwrap();

        let mut sum = 0;
        expected_prefix = expected_prices
            .iter()
            .map(|p| {
                sum += p.inner();
                StoredU64::from(sum)
            })
            .collect();
        let expected_sma: Vec<_> = (0..end)
            .map(|i| {
                let from = i.saturating_sub(73);
                let sum: u64 = expected_prices[from..=i].iter().map(|p| p.inner()).sum();
                Cents::new(sum / (i - from + 1) as u64)
            })
            .collect();

        for cold in [false, true] {
            if cold {
                budget.clear();
            }
            assert_eq!(reader.collect(), expected_prefix);
            assert_eq!(sma.collect(), expected_sma);
            assert_eq!(sma.collect_one_at(end - 1), expected_sma.last().copied());
            let indices = [0, 255, 256, 256, end - 1, end];
            assert_eq!(
                sma.read_sorted_at(&indices),
                indices
                    .iter()
                    .filter_map(|&i| expected_sma.get(i).copied())
                    .collect::<Vec<_>>()
            );
            assert_eq!(sma.collect_range_at(250, 270), expected_sma[250..270]);
        }
    }
    drop((sma, reader, starts, prices, prefix, db));
    let db = Database::open(directory.path()).unwrap();
    let prefix = import_cached::<Height, StoredU64>(&db, "prefix", Version::ONE).unwrap();
    assert_eq!(prefix.collect(), expected_prefix);
}
