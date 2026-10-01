use bitview_cohort::{AgeRange, AgeRangeId, AmountRange, CohortContext};
use bitview_vecs::DisjointAgeSources;
use brk_types::{CentsSats, CentsSquaredSats, Height, Version};
use tempfile::tempdir;
use vecdb::{AnyStoredVec, BytesVec, Database, ImportableVec, ReadableVec, WritableVec};

fn raw(band: usize, row: usize) -> (u128, u128, u128) {
    let price = u32::MAX as u128 + 17 + row as u128;
    let sats = (band as u128 + 1) * 1_000_000_003;
    (
        price * sats + (price + 9) * 2,
        price * price * sats + (price + 9).pow(2) * 2,
        sats + 2,
    )
}

#[test]
fn raw_age_caps_preserve_every_threshold_price_through_rollback_and_reopen() {
    let directory = tempdir().unwrap();
    {
        let db = Database::open(directory.path()).unwrap();
        let mut caps = DisjointAgeSources::forced_import(&db, "cap_raw", Version::ONE).unwrap();
        let mut capitals =
            DisjointAgeSources::forced_import(&db, "capitalized_cap_raw", Version::ONE).unwrap();
        for row in 0..3 {
            caps.push_age(&AgeRange::from_fn(|id| {
                CentsSats::new(raw(id.index(), row).0)
            }));
            capitals.push_age(&AgeRange::from_fn(|id| {
                CentsSquaredSats::new(raw(id.index(), row).1)
            }));
        }
        assert_eq!(caps.len(), 3);
        assert_eq!(capitals.len(), 3);
        assert_eq!(caps.collect_vecs_mut().len(), AgeRangeId::ALL.len());
        for source in caps
            .collect_vecs_mut()
            .into_iter()
            .chain(capitals.collect_vecs_mut())
        {
            source.write().unwrap();
            source.any_truncate_if_needed_at(1).unwrap();
        }
        caps.push_age(&AgeRange::from_fn(|id| {
            CentsSats::new(raw(id.index(), 7).0)
        }));
        capitals.push_age(&AgeRange::from_fn(|id| {
            CentsSquaredSats::new(raw(id.index(), 7).1)
        }));
        for source in caps
            .collect_vecs_mut()
            .into_iter()
            .chain(capitals.collect_vecs_mut())
        {
            source.write().unwrap();
        }
        db.flush().unwrap();
    }
    let db = Database::open(directory.path()).unwrap();
    let caps =
        DisjointAgeSources::<CentsSats>::forced_import(&db, "cap_raw", Version::ONE).unwrap();
    let capitals = DisjointAgeSources::<CentsSquaredSats>::forced_import(
        &db,
        "capitalized_cap_raw",
        Version::ONE,
    )
    .unwrap();
    assert_eq!(caps.len(), 2);
    assert_eq!(capitals.len(), 2);
    for (height, row) in [0, 7].into_iter().enumerate() {
        for split in 1..AgeRangeId::ALL.len() {
            for ids in [&AgeRangeId::ALL[..split], &AgeRangeId::ALL[split..]] {
                let (mut cap, mut capital) = (0u128, 0u128);
                let (mut expected_cap, mut expected_capital, mut supply) = (0u128, 0u128, 0u128);
                for id in ids {
                    cap += id
                        .select(&caps.age)
                        .collect_one_at(height)
                        .unwrap()
                        .as_u128();
                    capital += id
                        .select(&capitals.age)
                        .collect_one_at(height)
                        .unwrap()
                        .inner();
                    let (rcap, ccap, sats) = raw(id.index(), row);
                    expected_cap += rcap;
                    expected_capital += ccap;
                    supply += sats;
                }
                assert_eq!(cap, expected_cap);
                assert_eq!(capital, expected_capital);
                assert_eq!(cap / supply, expected_cap / supply);
                assert_eq!(capital / cap, expected_capital / expected_cap);
            }
        }
    }
}

#[test]
fn raw_amount_buckets_keep_the_remainder_needed_for_aggregate_realized_prices() {
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut sources = AmountRange::try_new(|id| {
        BytesVec::<Height, CentsSats>::forced_import(
            &db,
            &CohortContext::Utxo.metric_name(id, "cap_raw"),
            Version::ONE,
        )
    })
    .unwrap();
    for (band, source) in sources.iter_mut().enumerate() {
        source.push(CentsSats::new(raw(band, band).0));
        source.write().unwrap();
    }
    let sources: Vec<_> = sources.iter().collect();
    let mut loses_precision = false;
    for split in 1..sources.len() {
        for range in [0..split, split..sources.len()] {
            let (mut raw_cap, mut supply, mut rounded_cap) = (0u128, 0u128, 0u128);
            for band in range {
                let cap = sources[band].collect_one_at(0).unwrap().as_u128();
                let (expected, _, sats) = raw(band, band);
                assert_eq!(cap, expected);
                raw_cap += cap;
                rounded_cap += cap / 100_000_000 * 100_000_000;
                supply += sats;
            }
            loses_precision |= raw_cap / supply != rounded_cap / supply;
        }
    }
    assert!(loses_precision);
}
