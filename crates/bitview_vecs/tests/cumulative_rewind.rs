use bitview_collections::Windows;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_types::{Height, StoredU64, Version};
use common::init_cache;
use tempfile::tempdir;
use vecdb::{AnyStoredVec, Database, ReadableVec, WritableVec};

mod common;

#[test]
fn mutable_checkpoint_access_invalidates_same_length_cumulative_state() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = common::indexes(&db);
    let starts = common::stored::<Height, _>(&db, "starts", [Height::ZERO; 4]);
    let windows = Windows {
        _24h: &starts,
        _1w: &starts,
        _1m: &starts,
        _1y: &starts,
    };
    let mut values = PerBlockCumulativeRolling::<StoredU64>::forced_import(
        &db,
        "values",
        Version::ONE,
        &indexes,
        &windows,
    )
    .unwrap();
    values.push_block(StoredU64::from(2_u64));
    values.push_block(StoredU64::from(3_u64));
    values.write().unwrap();
    assert_eq!(values.block.collect(), [2_u64, 3].map(StoredU64::from));

    values.stored_mut().any_truncate_if_needed_at(1).unwrap();
    values.cumulative.height.push(StoredU64::from(12_u64));
    values.cumulative.height.write().unwrap();
    values.push_block(StoredU64::from(1_u64));
    values.write().unwrap();
    assert_eq!(
        values.cumulative.height.collect(),
        [2_u64, 12, 13].map(StoredU64::from)
    );
    assert_eq!(values.block.collect(), [2_u64, 10, 1].map(StoredU64::from));
}
