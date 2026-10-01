use bitview_cohort::*;

#[test]
fn iteration_and_mutation_follow_cohort_order() {
    macro_rules! check {
        ($collection:ident, $id:ident) => {{
            let mut values = $collection::from_fn(|id| id as usize);
            assert!(values.iter().copied().eq(0..$id::ALL.len()));
            for value in values.iter_mut() {
                *value += 1;
            }
            for &id in $id::ALL {
                assert_eq!(*id.select(&values), id as usize + 1);
            }
        }};
    }
    check!(AgeRange, AgeRangeId);
    check!(SpendableType, SpendableTypeId);
    check!(AmountRange, AmountRangeId);
    check!(ByEntry, EntryPrice);
    check!(ByEpoch, EpochId);
    check!(Class, ClassId);
    check!(ProfitabilityRange, ProfitabilityRangeId);
    check!(UTXOAggregate, UTXOAggregateId);
    check!(UTXOAllAndSth, UTXOAllAndSthId);

    let mut terms = ByTerm::from_fn(|term| term);
    assert!(terms.iter().copied().eq([Term::Sth, Term::Lth]));
    assert_eq!(*terms.get(Term::Sth), Term::Sth);
    *terms.get_mut(Term::Lth) = Term::Sth;
    assert!(terms.iter().all(|&term| term == Term::Sth));
}

#[test]
fn fallible_construction_stops_at_the_first_error() {
    let mut calls = 0;
    let result = AmountRange::try_new(|id| {
        calls += 1;
        if calls == 2 { Err(id.name()) } else { Ok(()) }
    });
    assert_eq!(
        result.err(),
        Some(AMOUNT_RANGE_NAMES.iter().nth(1).unwrap().id)
    );
    assert_eq!(calls, 2);
}
