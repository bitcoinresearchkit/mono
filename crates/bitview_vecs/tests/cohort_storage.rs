use common::init_cache;

use bitview_cohort::{
    AgeRange, AmountRange, CohortContext, CohortId, UTXOAggregate, UTXOCoreValues,
};
use bitview_traversable::Traversable;
use bitview_vecs::{
    AggregateFiatPerBlock, AggregatePerBlock, AggregatePercentPerBlock,
    AggregatePriceWithRatioPerBlock, AmountSources, CreationSources,
};
use brk_types::{Cents, Height, PartsPerMillion32, StoredU64, Version};
use tempfile::tempdir;
use vecdb::{Database, PcoVecValue, ReadOnlyClone, ReadableCloneableVec, ReadableVec, Ro};

mod common;

#[test]
fn aggregate_view_families_share_storage_and_read_only_projection() {
    init_cache();
    fn check<V: Clone, T: PcoVecValue + PartialEq>(
        mut owner: AggregatePerBlock<V, T>,
        values: UTXOAggregate<T>,
    ) where
        AggregatePerBlock<V, T>: Traversable,
        AggregatePerBlock<V, T, Ro>: Traversable,
    {
        assert!(owner.is_empty());
        owner.push(values.clone());
        assert_eq!(owner.len(), 1);
        let stored = owner.collect_vecs_mut();
        assert_eq!(stored.len(), 3);
        for source in stored {
            source.write().unwrap();
        }
        let reader = owner.read_only_clone();
        for (source, expected) in reader.stored.iter().zip(values.iter()) {
            assert_eq!(source.collect_one(Height::ZERO), Some(*expected));
        }
        assert_eq!(owner.to_tree_node(), reader.to_tree_node());
        assert_eq!(
            owner.iter_any_exportable().count(),
            owner.iter_any_visible().count() + 3,
        );
    }

    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = common::indexes(&db);
    let spot = common::stored::<Height, _>(&db, "spot", [Cents::from(100_u64)]);
    let amounts = UTXOAggregate::from_fn(|id| Cents::from(id.index() as u64 + 1));
    check(
        AggregateFiatPerBlock::forced_import(&db, "fiat", Version::ONE, &indexes).unwrap(),
        amounts.clone(),
    );
    check(
        AggregatePercentPerBlock::forced_import(&db, "share", Version::ONE, &indexes).unwrap(),
        UTXOAggregate::from_fn(|id| PartsPerMillion32::from(id.index() as f64 / 2.0)),
    );
    check(
        AggregatePriceWithRatioPerBlock::forced_import(
            &db,
            "price",
            Version::ONE,
            &indexes,
            &spot.read_only_boxed_clone(),
        )
        .unwrap(),
        amounts,
    );
}

#[test]
fn creation_cohorts_reopen_without_storing_holder_aggregates() {
    init_cache();
    let dir = tempdir().unwrap();
    let version = Version::new(31);
    let mut expected_names = Vec::new();
    {
        let db = Database::open(dir.path()).unwrap();
        let mut sources =
            CreationSources::<StoredU64>::forced_import(&db, "exact", version).unwrap();
        let mut direct = UTXOCoreValues::default();
        direct.age_range = AgeRange::from_fn(|id| StoredU64::from(id.index() as u64 + 1));
        sources.push(direct);
        for vec in sources.collect_vecs_mut() {
            assert_eq!(vec.len(), 1);
            expected_names.push(vec.name().to_owned());
            vec.write().unwrap();
        }
        db.flush().unwrap();
    }
    let db = Database::open(dir.path()).unwrap();
    let mut sources = CreationSources::<StoredU64>::forced_import(&db, "exact", version).unwrap();
    assert_eq!(sources.min_len(), 1);
    let names: Vec<_> = sources
        .collect_vecs_mut()
        .iter()
        .map(|v| v.name().to_owned())
        .collect();
    assert_eq!(names, expected_names);
    assert!(sources.get(CohortId::All).is_none());
    let values = sources.collect_last().unwrap();
    for (index, value) in values.age_range.iter().enumerate() {
        assert_eq!(*value, StoredU64::from(index as u64 + 1));
    }
}

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
    assert_eq!(amounts.len(), 3);
    assert!(
        amounts
            .checkpoint(Height::from(2_usize))
            .unwrap()
            .iter()
            .all(|v| *v == StoredU64::from(13_u64))
    );
}
