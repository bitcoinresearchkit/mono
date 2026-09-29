use std::sync::atomic::{AtomicUsize, Ordering};

use brk_types::{Cents, Height, PartsPerMillion32, Version};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, Database, LazyVec, ReadableCloneableVec, ReadableVec, WritableVec,
};

use crate::{component, test_common as common};

static READS: AtomicUsize = AtomicUsize::new(0);

fn counted_price(_: Height, price: Cents) -> Cents {
    READS.fetch_add(1, Ordering::Relaxed);
    price
}

#[test]
fn boundaries_share_reference_reads_and_follow_source_revisions() {
    common::init_cache();
    let dir = tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let indexes = common::indexes(&db);
    let len = 14_003;
    let values = [
        Cents::ZERO,
        Cents::NAN,
        Cents::new(1_234_567),
        Cents::new(9_876_543),
    ];
    let mut source =
        common::stored::<Height, _>(&db, "reference", (0..len).map(|i| values[i % values.len()]));
    let counted = LazyVec::init(
        "counted",
        Version::ONE,
        source.read_only_boxed_clone(),
        counted_price,
    );
    let mut component =
        component::forced_import(&db, "test", Version::ONE, &indexes, &counted).unwrap();
    for (band, ratio) in component.ratios.iter_mut().enumerate() {
        for i in 0..len - band {
            ratio.push(PartsPerMillion32::from((i % 43_001) as f64 / 1000.0));
        }
        ratio.write().unwrap();
    }

    for (start, end) in [
        (0, len),
        (113, len - 27),
        (len - 5, len + 100),
        (len, len + 1),
        (3, 3),
    ] {
        READS.store(0, Ordering::Relaxed);
        let actual = component::collect_boundary_prices(&component, start, end);
        assert_eq!(
            READS.load(Ordering::Relaxed),
            end.min(source.len()).saturating_sub(start)
        );
        READS.store(0, Ordering::Relaxed);
        let expected = component
            .bands
            .boundary_refs()
            .map(|band| band.price.cents.height.collect_range_at(start, end));
        assert_eq!(
            READS.load(Ordering::Relaxed),
            expected.iter().map(Vec::len).sum::<usize>()
        );
        assert_eq!(actual, expected);
    }

    for new_len in [len + 9, len - 29, len + 13] {
        let from = source.len().min(new_len).saturating_sub(3);
        source.truncate_if_needed_at(from).unwrap();
        for _ in from..new_len {
            source.push(Cents::new(1_234_569));
        }
        source.write().unwrap();
        for ratio in component.ratios.iter_mut() {
            let ratio_from = ratio.len().min(from);
            ratio.truncate_if_needed_at(ratio_from).unwrap();
            for _ in ratio_from..new_len {
                ratio.push(PartsPerMillion32::from(1.001));
            }
            ratio.write().unwrap();
        }
        let actual = component::collect_boundary_prices(&component, from, new_len + 5);
        let expected = component
            .bands
            .boundary_refs()
            .map(|band| band.price.cents.height.collect_range_at(from, new_len + 5));
        assert_eq!(actual, expected);
        assert_eq!(component::boundary_len(&component), new_len);
    }
}
