use bitview_collections::Windows;
use bitview_compute::prepare_computed;
use bitview_vecs::PerBlockCumulativeRolling;
use brk_exit::Exit;
use brk_types::{Height, StoredU32, StoredU64, TxIndex, Version};
use common::{init_cache, stored};
use tempfile::tempdir;
use vecdb::{AnyStoredVec, Database, ReadableVec, WritableVec};

mod common;

#[test]
fn computed_outputs_share_the_shortest_valid_prefix() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut left = stored::<Height, StoredU64>(&db, "left", []);
    let mut right = stored::<Height, StoredU32>(&db, "right", []);
    for (left_len, right_len, right_version, max_from, expected) in [
        (5, 3, Version::ONE, 9, 3),
        (3, 5, Version::ONE, 9, 3),
        (5, 5, Version::ONE, 2, 2),
        (5, 5, Version::TWO, 9, 0),
        (0, 5, Version::ONE, 9, 0),
        (5, 5, Version::ONE, 0, 0),
        (5, 5, Version::ONE, 5, 5),
    ] {
        left.validate_computed_version_or_reset(Version::ONE)
            .unwrap();
        right
            .validate_computed_version_or_reset(right_version)
            .unwrap();
        left.truncate_if_needed_at(0).unwrap();
        right.truncate_if_needed_at(0).unwrap();
        for value in 0..left_len {
            left.push(StoredU64::from(value as u64));
        }
        for value in 0..right_len {
            right.push(StoredU32::from(value as u32));
        }
        left.write().unwrap();
        right.write().unwrap();
        assert_eq!(
            prepare_computed(
                [&mut left as &mut dyn AnyStoredVec, &mut right],
                Version::ONE,
                max_from,
            )
            .unwrap(),
            expected,
        );
        assert_eq!(
            left.collect(),
            (0..expected)
                .map(|n| StoredU64::from(n as u64))
                .collect::<Vec<_>>()
        );
        assert_eq!(
            right.collect(),
            (0..expected)
                .map(|n| StoredU32::from(n as u32))
                .collect::<Vec<_>>()
        );
    }
}

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

#[test]
fn grouped_totals_resume_through_empty_groups_and_refresh_the_append_cache() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = common::indexes(&db);
    let starts = stored::<Height, _>(&db, "starts", [Height::ZERO; 5]);
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
    let first = stored::<Height, _>(&db, "first", [0usize, 0, 2, 2].map(TxIndex::from));
    // The unpaired count must not create another block.
    let counts = stored::<Height, _>(&db, "counts", [0_u64, 2, 0, 1, 1].map(StoredU64::from));
    let amounts = stored::<TxIndex, _>(&db, "amounts", [2_u32, 3, 4, 9].map(StoredU32::from));
    let exit = Exit::new();
    for from in [Height::ZERO, Height::from(2usize)] {
        values
            .compute_cumulative_sum_from_indexes(
                from,
                &first,
                &counts,
                &amounts,
                |amount| StoredU64::from(u64::from(u32::from(amount)) * 10),
                &exit,
            )
            .unwrap();
        assert_eq!(
            values.cumulative.height.collect(),
            [0_u64, 50, 50, 90].map(StoredU64::from)
        );
        values.push_block(StoredU64::from(5_u64));
        values.write().unwrap();
        assert_eq!(
            values.cumulative.height.collect_last(),
            Some(StoredU64::from(95_u64))
        );
    }
}
