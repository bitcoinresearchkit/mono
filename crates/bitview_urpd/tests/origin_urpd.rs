use std::{array, collections::BTreeMap, slice};

use bitview_cohort::{AGE_BOUNDARIES, AgeAggregateId, AgeRange, AgeRangeId};
use bitview_urpd::{COST_BASIS_PRICE_DIGITS, OriginUrpd};
use brk_types::{Age, Cents, CentsCompact, ONE_HOUR_IN_SEC, Sats, Timestamp};
use statedb::{Amount, Creations, History, Reader, Spends, State};
use tempfile::tempdir;

fn project(
    entries: impl IntoIterator<Item = (AgeRangeId, CentsCompact, Sats)>,
    weights: &AgeRange<f64>,
) -> Box<[(CentsCompact, Sats)]> {
    let mut buckets = BTreeMap::<_, f64>::new();
    for (age, price, sats) in entries {
        *buckets.entry(price).or_default() += u64::from(sats) as f64 * *age.select(weights);
    }
    buckets
        .into_iter()
        .filter_map(|(price, mass)| {
            let sats = Sats::new(mass.floor() as u64);
            (sats != Sats::ZERO).then_some((price, sats))
        })
        .collect()
}

// Independent oracle: rebuild from origin amounts with ordered maps, rather than
// incrementally updating the production histogram and its stable bucket slots.
fn rebuild(
    state: &State,
    prices: &[Cents],
    timestamps: &[Timestamp],
) -> AgeRange<BTreeMap<CentsCompact, Sats>> {
    let mut groups = AgeRange::<BTreeMap<CentsCompact, Sats>>::default();
    if let Some(&current) = timestamps.get(state.len().wrapping_sub(1)) {
        for (h, amount) in state.amounts().iter().enumerate() {
            if amount.sats != 0 {
                let age = AgeRangeId::from(Age::new(current, timestamps[h]));
                let price = CentsCompact::from(prices[h]).round_to_dollar(COST_BASIS_PRICE_DIGITS);
                *age.select_mut(&mut groups).entry(price).or_default() += Sats::new(amount.sats);
            }
        }
    }
    groups
}

fn entries(
    groups: &AgeRange<BTreeMap<CentsCompact, Sats>>,
) -> impl Iterator<Item = (AgeRangeId, CentsCompact, Sats)> + '_ {
    AgeRangeId::ALL
        .iter()
        .copied()
        .flat_map(|age| age.select(groups).iter().map(move |(&p, &s)| (age, p, s)))
}

fn hash(h: usize) -> [u8; 32] {
    let mut hash = [0; 32];
    hash[..8].copy_from_slice(&(h as u64).to_le_bytes());
    hash
}

fn verify(
    reader: &Reader<'_>,
    prices: &[Cents],
    timestamps: &[Timestamp],
    start: usize,
    warm: bool,
) {
    let mut state = reader.state_at(start).unwrap();
    let available = if warm { start } else { prices.len() };
    let mut incremental =
        OriginUrpd::new(&state, &prices[..available], &timestamps[..available]).unwrap();
    let mut cursor = reader.cursor(&mut state).unwrap();
    for h in start..reader.len() {
        if warm {
            incremental.extend_prices(&prices[h..h + 1]).unwrap();
        }
        incremental.advance(&mut cursor, timestamps).unwrap();
        let direct = rebuild(cursor.state(), prices, timestamps);
        for &age in AgeRangeId::ALL {
            let actual: Vec<_> = incremental
                .project(&[], [slice::from_ref(&age)])
                .map(|row| (row.price, row.raw[0]))
                .collect();
            assert_eq!(
                actual,
                age.select(&direct)
                    .iter()
                    .map(|(&p, &s)| (p, s))
                    .collect::<Vec<_>>(),
                "height={h} start={start} age={age:?}"
            );
        }
        let raw = project(entries(&direct), &AgeRange::from_fn(|_| 1.0));
        assert_eq!(
            incremental
                .project(&[], [AgeAggregateId::All.age_range_ids()])
                .map(|b| (b.price, b.raw[0]))
                .collect::<Vec<_>>(),
            raw.as_ref()
        );
        let zero = AgeRange::from_fn(|_| 0.0);
        assert!(
            incremental
                .project(&[Some(&zero), None], [AgeAggregateId::All.age_range_ids()])
                .all(|b| b.weighted.iter().all(|mass| *mass == [Sats::ZERO; 2]))
        );
        let cohorts: [_; AgeAggregateId::ALL.len()] =
            array::from_fn(|i| AgeAggregateId::ALL[i].age_range_ids());
        for offset in 0..3 {
            let weights =
                AgeRange::from_fn(|age| ((age.index() * 7 + offset * 3) % 17) as f64 / 17.0);
            let selected = [Some(&weights)];
            let projected: Vec<_> = incremental.project(&selected, cohorts).collect();
            for &age in AgeRangeId::ALL {
                let expected = project(entries(&direct).filter(|(a, _, _)| *a == age), &weights);
                let actual: Vec<_> = incremental
                    .project(&selected, [slice::from_ref(&age)])
                    .filter_map(|b| {
                        let sats = b.weighted[0][0];
                        (sats != Sats::ZERO).then_some((b.price, sats))
                    })
                    .collect();
                assert_eq!(actual, expected.as_ref(), "{age:?} at {h}");
            }
            for &id in AgeAggregateId::ALL {
                let expected = project(
                    entries(&direct).filter(|(age, _, _)| id.contains(*age)),
                    &weights,
                );
                let actual: Vec<_> = projected
                    .iter()
                    .filter_map(|b| {
                        let sats = b.weighted[id.index()][0];
                        (sats != Sats::ZERO).then_some((b.price, sats))
                    })
                    .collect();
                assert_eq!(actual, expected.as_ref(), "{id:?} at {h}");
                let single: Vec<_> = incremental
                    .project(&selected, [id.age_range_ids()])
                    .filter_map(|b| {
                        let sats = b.weighted[0][0];
                        (sats != Sats::ZERO).then_some((b.price, sats))
                    })
                    .collect();
                assert_eq!(actual, single);
            }
        }
        let total: u64 = incremental
            .project(&[], [AgeAggregateId::All.age_range_ids()])
            .map(|row| {
                assert_ne!(row.raw[0], Sats::ZERO);
                u64::from(row.raw[0])
            })
            .sum();
        assert_eq!(total, cursor.state().total().sats);
    }
}

#[test]
fn diff_pass_matches_full_rebuild_at_boundaries_on_restart_and_reorg() {
    let root = tempdir().unwrap();
    let mut spends = Spends::open(root.path()).unwrap();
    let mut creations = Creations::open(root.path()).unwrap();
    let mut history = History::open(root.path()).unwrap();
    history.set_snapshot_interval(7).unwrap();
    let mut timestamps = vec![
        Timestamp::new(0),
        Timestamp::new(0),
        Timestamp::new(3599),
        Timestamp::new(3600),
    ];
    for boundary in AGE_BOUNDARIES {
        let t = boundary as u32 * ONE_HOUR_IN_SEC;
        for value in [t - 1, t, t, t + 1] {
            timestamps.push(Timestamp::new(value));
        }
    }
    timestamps.sort();
    let prices: Vec<_> = (0..timestamps.len())
        .map(|h| Cents::new((h as u64 * 12345) % 731 + 100))
        .collect();
    for h in 0..timestamps.len() {
        let mut removed = vec![(h as u32, Amount { sats: 1, count: 1 })];
        if h >= 2 {
            removed.push(((h - 2) as u32, Amount { sats: 99, count: 1 }));
        }
        spends.push(hash(h), removed).unwrap();
        // Leave zero-value outputs alive so count and occupied-price semantics differ.
        creations
            .push(
                hash(h),
                Amount {
                    sats: 100,
                    count: 3,
                },
                None,
            )
            .unwrap();
    }
    spends.commit().unwrap();
    creations.commit().unwrap();
    history
        .advance(0, timestamps.len(), &spends, &creations, |_, _| Ok(()))
        .unwrap();
    let reader = history.reader(&spends, &creations).unwrap();
    for start in [0, 1, 3, 4, 19, timestamps.len() - 2] {
        for warm in [false, true] {
            verify(&reader, &prices, &timestamps, start, warm);
        }
    }
    drop(reader);

    let from = 10;
    spends.truncate(from).unwrap();
    creations.truncate(from).unwrap();
    for h in from..timestamps.len() {
        spends.push(hash(h + 1000), []).unwrap();
        let correction = (h == from).then_some(((from - 1) as u32, Amount { sats: 99, count: 2 }));
        creations
            .push(hash(h + 1000), Amount { sats: 33, count: 1 }, correction)
            .unwrap();
    }
    spends.commit().unwrap();
    creations.commit().unwrap();
    history
        .advance(from, timestamps.len(), &spends, &creations, |_, _| Ok(()))
        .unwrap();
    let reader = history.reader(&spends, &creations).unwrap();
    for start in [0, from, from + 1] {
        for warm in [false, true] {
            verify(&reader, &prices, &timestamps, start, warm);
        }
    }
}
