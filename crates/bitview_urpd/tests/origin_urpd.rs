use bitview_cohort::{AGE_BOUNDARIES, AgeRange, AgeRangeId};
use bitview_urpd::{AgeRangeUrpds, OriginUrpd, accumulate_masses, collect_mass};
use brk_types::{Cents, CentsCompact, ONE_HOUR_IN_SEC, Sats, Timestamp};
use statedb::{Amount, Creations, History, Reader, Spends};
use tempfile::tempdir;

fn project(
    entries: impl IntoIterator<Item = (AgeRangeId, CentsCompact, Sats)>,
    weights: &AgeRange<f64>,
) -> Box<[(CentsCompact, Sats)]> {
    let buckets = accumulate_masses(entries, |mass: &mut f64, age, sats| {
        *mass += u64::from(sats) as f64 * *age.select(weights);
    });
    collect_mass(&buckets, |&mass| mass)
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
        let direct = AgeRangeUrpds::from_origins(cursor.state(), prices, timestamps).unwrap();
        for &age in AgeRangeId::ALL {
            let actual: Vec<_> = incremental
                .iter()
                .filter(|(a, _, _)| *a == age)
                .map(|(_, p, s)| (p, s))
                .collect();
            assert_eq!(
                actual,
                direct.get(age),
                "height={h} start={start} age={age:?}"
            );
        }
        let raw = project(direct.iter(), &AgeRange::from_fn(|_| 1.0));
        assert_eq!(
            incremental
                .project(&[])
                .map(|b| (b.price, b.raw))
                .collect::<Vec<_>>(),
            raw.as_ref()
        );
        let zero = AgeRange::from_fn(|_| 0.0);
        assert!(
            incremental
                .project(&[Some(&zero), None])
                .all(|b| b.weighted == [Sats::ZERO; 2])
        );
        for offset in 0..3 {
            let weights =
                AgeRange::from_fn(|age| ((age.index() * 7 + offset * 3) % 17) as f64 / 17.0);
            let expected = project(direct.iter(), &weights);
            let selected = [Some(&weights)];
            let view = incremental
                .project(&selected)
                .filter_map(|b| (b.weighted[0] != Sats::ZERO).then_some((b.price, b.weighted[0])));
            assert_eq!(view.clone().collect::<Vec<_>>(), expected.as_ref());
            assert_eq!(view.collect::<Vec<_>>(), expected.as_ref());
        }
        let total: u64 = incremental.iter().map(|(_, _, s)| u64::from(s)).sum();
        assert_eq!(total, cursor.state().total().sats);
        assert!(incremental.iter().all(|(_, _, sats)| sats != Sats::ZERO));
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
