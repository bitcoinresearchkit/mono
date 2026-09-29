use common::init_cache;
use std::ptr;

use bitview_cohort::{
    AgeRange, AgeRangeId, AmountRange, AmountRangeId, CohortContext, CohortId, SpendableTypeId,
    Term, UTXOAggregate, UTXOValues,
};
use bitview_traversable::Traversable;
use bitview_vecs::{
    AggregateFiatPerBlock, AggregatePerBlock, AggregatePercentPerBlock,
    AggregatePriceWithRatioPerBlock, AmountSources, UTXOSources,
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
fn exact_totals_never_sum_independently_computed_values() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut sources =
        UTXOSources::<Cents>::forced_import(&db, "exact_prices", Version::ONE).unwrap();
    let maximum = Cents::from(u64::MAX - 1);
    let direct = UTXOValues::<Cents>::default().map(|_| maximum);
    let aggregate = UTXOAggregate::<Cents>::default().map(|_| Cents::from(17_u64));
    sources.push_exact(direct, aggregate);
    assert_eq!(sources.collect_vecs_mut().len(), 77);
    for source in sources.collect_vecs_mut() {
        source.write().unwrap();
    }
    assert_eq!(
        sources.get(CohortId::All).unwrap().collect_one_at(0),
        Some(Cents::from(17_u64))
    );
    assert!(
        AgeRangeId::ALL
            .iter()
            .all(|id| sources.get(id.cohort()).unwrap().collect_one_at(0) == Some(maximum))
    );
    assert!(
        sources
            .amount
            .iter()
            .all(|source| source.collect_one_at(0) == Some(maximum))
    );
    assert!([Term::Sth, Term::Lth].iter().all(|&term| {
        sources.get(CohortId::Term(term)).unwrap().collect_one_at(0) == Some(Cents::from(17_u64))
    }));
}

#[test]
fn source_selection_borrows_the_named_owner_for_each_cohort_family() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let sources = UTXOSources::<StoredU64>::forced_import(&db, "selection", Version::ONE).unwrap();
    for id in AmountRangeId::ALL {
        assert!(ptr::eq(
            sources.get(id.cohort()).unwrap(),
            id.select(&sources.amount)
        ));
    }
    for id in SpendableTypeId::ALL {
        assert!(ptr::eq(
            sources.get(CohortId::Type(id.output_type())).unwrap(),
            id.select(&sources.type_)
        ));
    }
}

#[test]
fn native_cohorts_reopen_and_preserve_independently_computed_totals() {
    init_cache();
    let dir = tempdir().unwrap();
    let version = Version::new(31);
    let mut expected_names = Vec::new();
    {
        let db = Database::open(dir.path()).unwrap();
        let mut sources = UTXOSources::<StoredU64>::forced_import(&db, "exact", version).unwrap();
        let mut direct = UTXOValues::default();
        direct.core.age_range = AgeRange::from_fn(|id| StoredU64::from(id.index() as u64 + 1));
        let aggregate = UTXOAggregate::default().map(|_: &StoredU64| StoredU64::from(17_u64));
        sources.push_exact(direct, aggregate);
        for vec in sources.collect_vecs_mut() {
            assert_eq!(vec.len(), 1);
            expected_names.push(vec.name().to_owned());
            vec.write().unwrap();
        }
        db.flush().unwrap();
    }
    let db = Database::open(dir.path()).unwrap();
    let mut sources = UTXOSources::<StoredU64>::forced_import(&db, "exact", version).unwrap();
    assert_eq!(sources.min_len(), 1);
    let names: Vec<_> = sources
        .collect_vecs_mut()
        .iter()
        .map(|v| v.name().to_owned())
        .collect();
    assert_eq!(names, expected_names);
    assert_eq!(
        sources.get(CohortId::All).unwrap().collect_one_at(0),
        Some(StoredU64::from(17_u64))
    );
    let values = sources.collect_last().unwrap();
    for (index, value) in values.core.age_range.iter().enumerate() {
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
