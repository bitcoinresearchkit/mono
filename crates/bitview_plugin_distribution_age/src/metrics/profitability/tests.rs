#[cfg(test)]
use crate::test_cache::init_cache;
use bitview_cohort::{ByTerm, ProfitabilityRange, ProfitabilityRangeId, Term, UTXOAggregateId};
use bitview_collections::Windows;
use bitview_plugin::ImportContext;
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, import_cached};
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::{Cents, CentsSats, Height, PartsPerMillionSigned32, Sats, Version};
use tempfile::tempdir;
use vecdb::{Database, ReadableCloneableVec, ReadableVec};

use super::ProfitabilityVecs;

#[test]
fn derived_values_preserve_profit_and_loss_polarity() {
    let supply = ProfitabilityRange::from_fn(|_| Sats::ONE_BTC);
    let cap = ProfitabilityRange::from_fn(|id| {
        Cents::from(if id.is_profit() { 100_u64 } else { 300_u64 })
    });
    let spot = Cents::from(200_u64);

    let cap = ByTerm {
        short: cap.clone(),
        long: cap,
    };
    let supply = ByTerm {
        short: supply.clone(),
        long: supply,
    };
    let pnl = ProfitabilityVecs::unrealized_pnl_by_term(spot, &cap, &supply);
    let all_cap = ProfitabilityVecs::sum_terms(&cap);
    let all_supply = ProfitabilityVecs::sum_terms(&supply);
    let nupl = ProfitabilityVecs::nupl(spot, &all_cap, &all_supply);

    for id in ProfitabilityRangeId::ALL {
        assert_eq!(*id.select(&pnl.short), Cents::from(100_u64));
        assert_eq!(*id.select(&pnl.long), Cents::from(100_u64));
    }
    for id in ProfitabilityRangeId::ALL {
        assert_eq!(
            *id.select(&nupl),
            PartsPerMillionSigned32::from(if id.is_profit() { 0.5 } else { -0.5 })
        );
    }
}

// Exercise the actual stored range sources, including empty buckets, sub-BTC
// amounts, both terms, and all 23 former aggregate selections.
#[test]
fn stored_ranges_reconstruct_removed_thresholds() {
    init_cache();
    let directory = tempdir().unwrap();
    let context = ImportContext::new(directory.path());
    let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
    let reader = Reader::new_without_rlimit(directory.path().join("blocks"), &client);
    let indexer = Indexer::import(context, &reader).unwrap();
    let mappings = MappingsVecs::import(context, &indexer).unwrap();
    let db = Database::open(&directory.path().join("profitability")).unwrap();
    let spot_source = import_cached::<Height, Cents>(&db, "spot", Version::ONE).unwrap();
    let starts = LazyWindowStartVec::days("window", Version::ONE, 1, &mappings.timestamp.monotonic);
    let windows = Windows {
        _24h: &starts,
        _1w: &starts,
        _1m: &starts,
        _1y: &starts,
    };
    let mut vecs = ProfitabilityVecs::forced_import(
        &db,
        Version::ONE,
        &mappings,
        &windows,
        &spot_source.read_only_boxed_clone(),
    )
    .unwrap();
    let spot = Cents::new(100_003);
    let supply = ByTerm {
        short: ProfitabilityRange::from_fn(|id| {
            Sats::new(if id.index() % 4 == 0 {
                0
            } else {
                123_457 * (id.index() as u64 + 1)
            })
        }),
        long: ProfitabilityRange::from_fn(|id| Sats::new(765_431 * (id.index() as u64 + 1))),
    };
    let cap_for = |supply: &ProfitabilityRange<Sats>| {
        ProfitabilityRange::from_fn(|id| {
            let price = if id.is_profit() {
                123 + id.index() as u64 * 500
            } else {
                120_000 + id.index() as u64 * 10_000
            };
            CentsSats::from_price_sats(Cents::new(price), *id.select(supply)).to_cents_rounded()
        })
    };
    let cap = ByTerm {
        short: cap_for(&supply.short),
        long: cap_for(&supply.long),
    };
    let pnl = ProfitabilityVecs::unrealized_pnl_by_term(spot, &cap, &supply);
    vecs.push(spot, supply.clone(), cap.clone());
    // A second, empty block checks zero-supply NUPL and resume tracking.
    vecs.push(Cents::ZERO, ByTerm::default(), ByTerm::default());
    let sources = vecs.collect_all_vecs_mut();
    assert_eq!(sources.len(), 25 * (3 * 3 + 1));
    for source in sources {
        source.write().unwrap();
    }
    assert_eq!(vecs.min_resume_len(), 2);

    for selection in (2..=15)
        .map(|end| 0..end)
        .chain((15..=23).map(|start| start..25))
    {
        for &term in UTXOAggregateId::ALL {
            let mut stored_supply = 0_u128;
            let mut stored_cap = 0_u128;
            let mut stored_pnl = 0_u128;
            let mut expected_supply = 0_u128;
            let mut expected_cap = 0_u128;
            let mut expected_pnl = 0_u128;
            for index in selection.clone() {
                let id = ProfitabilityRangeId::ALL[index];
                stored_supply += term
                    .select(id.select(&vecs.supply_stored))
                    .collect_one_at(0)
                    .unwrap()
                    .as_u128();
                stored_cap += term
                    .select(id.select(&vecs.realized_cap_stored))
                    .collect_one_at(0)
                    .unwrap()
                    .as_u128();
                stored_pnl += term
                    .select(id.select(&vecs.unrealized_pnl_stored))
                    .collect_one_at(0)
                    .unwrap()
                    .as_u128();
                let terms = match term {
                    UTXOAggregateId::All => &[Term::Sth, Term::Lth][..],
                    UTXOAggregateId::Sth => &[Term::Sth][..],
                    UTXOAggregateId::Lth => &[Term::Lth][..],
                };
                for &term in terms {
                    expected_supply += id.select(supply.get(term)).as_u128();
                    expected_cap += id.select(cap.get(term)).as_u128();
                    expected_pnl += id.select(pnl.get(term)).as_u128();
                }
            }
            assert_eq!(
                (stored_supply, stored_cap, stored_pnl),
                (expected_supply, expected_cap, expected_pnl)
            );
            // Reproduce the removed NUPL calculation from additive range data,
            // retaining its integer realized-price truncation before ppm rounding.
            let nupl = |cap: u128, supply: u128| {
                if supply == 0 {
                    return PartsPerMillionSigned32::ZERO;
                }
                let price = cap * Sats::ONE_BTC_U128 / supply;
                PartsPerMillionSigned32::from(
                    (spot.as_u128() as f64 - price as f64) / spot.as_u128() as f64,
                )
            };
            assert_eq!(
                nupl(stored_cap, stored_supply),
                nupl(expected_cap, expected_supply)
            );
        }
    }
    for source in vecs.nupl_stored.iter() {
        assert_eq!(
            source.collect_one_at(1),
            Some(PartsPerMillionSigned32::ZERO)
        );
    }
}
