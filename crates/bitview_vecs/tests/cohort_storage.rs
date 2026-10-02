use cache::init_cache;

use bitview_cohort::{AmountRange, CohortContext};
use bitview_vecs::AmountSources;
use brk_types::{Height, StoredU64, Version};
use tempfile::tempdir;
use vecdb::Database;

#[path = "common/cache.rs"]
mod cache;

#[test]
fn amount_composition_keeps_checkpoint_invalidation_and_reader_projection() {
    init_cache();
    let dir = tempdir().unwrap();
    let db = Database::open(dir.path()).unwrap();
    let mut amounts = AmountSources::<StoredU64, ()>::forced_import(
        &db,
        "amount_source",
        CohortContext::Addr,
        "amount",
        Version::ONE,
        |_, _| (),
    )
    .unwrap();
    let values = |value| AmountRange::from_fn(|_| StoredU64::from(value));
    for value in [2_u64, 3] {
        amounts.push_cumulative(&values(value));
    }
    for vec in amounts.stored_vecs_mut() {
        vec.write().unwrap();
        vec.any_truncate_if_needed_at(1).unwrap();
    }
    amounts.push_cumulative(&values(10));
    amounts.push_cumulative(&values(1));
    for vec in amounts.stored_vecs_mut() {
        vec.write().unwrap();
    }
    assert_eq!(amounts.min_len(), 3);
    assert!(
        amounts
            .checkpoint(Height::from(2_usize))
            .unwrap()
            .iter()
            .all(|v| *v == StoredU64::from(13_u64))
    );
}
