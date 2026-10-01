use bitview_cohort::*;
use brk_types::OutputType;

#[test]
fn identity_roundtrips_through_composed_groups() {
    let cohorts = UTXOGroups::new(|id| id);
    assert_eq!(cohorts.iter().count(), 75);
    for &id in cohorts.iter() {
        assert_eq!(cohorts.get(id), Some(&id));
    }
    cohorts.map_with_id(|id, &value| assert_eq!(id, value));
    assert_eq!(cohorts.get(CohortId::Type(OutputType::OpReturn)), None);

    let core = UTXOGroupCore::new(|id| id);
    for &id in core.iter() {
        assert_eq!(core.get(id), Some(&id));
    }
    assert_eq!(core.get(CohortId::Term(Term::Sth)), None);
    assert_eq!(core.get(CohortId::Entry(EntryPrice::Discount)), None);
    assert_eq!(core.get(AmountRangeId::Zero.cohort()), None);
    assert_eq!(core.get(CohortId::Type(OutputType::P2PKH)), None);
}

#[test]
fn holder_classification_matches_age_bounds_and_aggregate_selectors() {
    for &id in AgeRangeId::ALL {
        let bounds = id.bounds();
        match id.term() {
            Term::Sth => assert!(bounds.end <= Term::THRESHOLD_HOURS),
            Term::Lth => assert!(bounds.start >= Term::THRESHOLD_HOURS),
        }
    }
    for &id in UTXOAggregateId::ALL {
        assert_eq!(
            id.cohort().age_ranges().unwrap().collect::<Vec<_>>(),
            id.age_range_ids(),
        );
    }
}

#[test]
fn composed_mapping_does_not_fall_back_to_the_dereferenced_core() {
    let cohorts = UTXOGroupsWithoutAmountOrType::new(|id| id);
    let mut count = 0;
    let mapped = cohorts.map_with_id(|id, &value| {
        count += 1;
        assert_eq!(id, value);
        value
    });
    assert_eq!(count, cohorts.iter().count());
    assert_eq!(mapped.term.short, CohortId::Term(Term::Sth));
    assert_eq!(mapped.term.long, CohortId::Term(Term::Lth));
}

#[test]
fn canonical_names_preserve_series_prefixes() {
    for (id, name) in [
        (CohortId::All, "supply"),
        (CohortId::Term(Term::Sth), "sth_supply"),
        (CohortId::Term(Term::Lth), "lth_supply"),
        (AgeRangeId::Under1H.cohort(), "utxos_under_1h_old_supply"),
        (AgeRangeId::Over15Y.cohort(), "utxos_over_15y_old_supply"),
        (AmountRangeId::Zero.cohort(), "utxos_0sats_supply"),
        (EpochId::_0.cohort(), "epoch_0_supply"),
        (ClassId::_2009.cohort(), "class_2009_supply"),
        (CohortId::Entry(EntryPrice::Discount), "veteran_supply"),
        (CohortId::Entry(EntryPrice::Premium), "rookie_supply"),
        (
            CohortId::Type(OutputType::Unknown),
            "unknown_outputs_supply",
        ),
        (CohortId::Type(OutputType::Empty), "empty_outputs_supply"),
        (CohortId::Type(OutputType::OpReturn), "op_return_supply"),
    ] {
        assert_eq!(CohortContext::Utxo.metric_name(id, "supply"), name);
    }
    assert_eq!(
        CohortContext::Addr.metric_name(AmountRangeId::Zero.cohort(), "supply"),
        "addrs_0sats_supply"
    );
}

#[test]
fn amount_names_match_cohort_names() {
    for &id in AmountRangeId::ALL {
        assert_eq!(id.name().id, id.cohort().name());
    }
}
