use std::{sync::Once, thread};

use bitview_cohort::{AgeAggregateId, AgeRangeId};
use bitview_collections::Windows;
use bitview_plugin::{ComputePlugin, ImportContext, UpdateContext};
use bitview_plugin_distribution_age::{Dependencies as AgeDependencies, Vecs as AgeVecs};
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_plugin_price::Vecs as Price;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyWindowStartVec, import_cached};
use brk_error::Result;
use brk_exit::Exit;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::{
    Age, Cents, CentsCompact, Height, PERCENTILES, PartsPerMillionSigned32, Sats, Timestamp,
    Version,
};
use statedb::{Amount, Creations, History, Spends};
use tempfile::{TempDir, tempdir};
use vecdb::{
    AnyStoredVec, AnyVec, Budgeted, Database, ReadOnlyClone, ReadableCloneableVec, ReadableVec,
    WritableVec,
};

use crate::{Dependencies, Vecs};

struct Fixture {
    directory: TempDir,
    _indexer: Indexer,
    db: Database,
    mappings: Mappings,
    price: Price,
    supply: CachedSeries<Height, Sats>,
    spends: Spends,
    creations: Creations,
    history: History,
}

impl Fixture {
    fn new() -> Self {
        static INIT: Once = Once::new();
        INIT.call_once(|| {
            Budgeted::init_global(2 * 1024 * 1024 * 1024).unwrap();
        });
        let directory = tempdir().unwrap();
        let context = ImportContext::new(directory.path());
        let client = Client::new("http://127.0.0.1:1", Auth::None).unwrap();
        let reader = Reader::new_without_rlimit(directory.path().join("blocks"), &client);
        let indexer = Indexer::import(context, &reader).unwrap();
        let mappings = Mappings::import(context, &indexer).unwrap();
        let price = Price::import(context, &mappings).unwrap();
        let db = Database::open(&directory.path().join("sources")).unwrap();
        let path = directory.path().join("history");
        Self {
            supply: import_cached(&db, "supply", Version::ONE).unwrap(),
            spends: Spends::open(&path).unwrap(),
            creations: Creations::open(&path).unwrap(),
            history: History::open(&path).unwrap(),
            directory,
            mappings,
            price,
            _indexer: indexer,
            db,
        }
    }

    fn import(&self) -> (AgeVecs, Vecs) {
        let starts = LazyWindowStartVec::days(
            "window",
            Version::ONE,
            1,
            &self.mappings.timestamp.monotonic,
        );
        let windows = Windows {
            _24h: &starts,
            _1w: &starts,
            _1m: &starts,
            _1y: &starts,
        };
        let context = ImportContext::new(self.directory.path());
        let supply = self.supply.read_only_boxed_clone();
        (
            AgeVecs::import(context, &self.mappings, &windows, &self.price, &supply).unwrap(),
            Vecs::import(context, &self.mappings, &windows, &self.price, &supply).unwrap(),
        )
    }

    fn append(
        &mut self,
        day: u32,
        price: u64,
        created: Amount,
        removed: &[(u32, Amount)],
        fork: u8,
    ) {
        let mut hash = [fork; 32];
        hash[..8].copy_from_slice(&(self.creations.end() as u64).to_le_bytes());
        self.spends.push(hash, removed.iter().copied()).unwrap();
        self.creations.push(hash, created, None).unwrap();
        self.price.spot.cents.height.push(Cents::new(price));
        self.mappings
            .timestamp
            .monotonic
            .push(Timestamp::new(1_231_006_505 + day * 86400));
    }

    fn publish(&mut self, from: usize) {
        self.spends.commit().unwrap();
        self.creations.commit().unwrap();
        self.supply.truncate_if_needed_at(from).unwrap();
        self.history
            .advance(
                from,
                self.creations.end(),
                &self.spends,
                &self.creations,
                |_, state| {
                    self.supply.push(Sats::new(state.sats));
                    Ok(())
                },
            )
            .unwrap();
        self.supply.write().unwrap();
        self.price.spot.cents.height.write().unwrap();
        self.mappings.timestamp.monotonic.write().unwrap();
    }

    fn compute(&self, age: &mut AgeVecs, plugin: &mut Vecs, from: usize) -> Result<()> {
        let reader = self.history.reader(&self.spends, &self.creations)?;
        age.compute(
            AgeDependencies {
                history: &reader,
                from: Height::from(from),
                mappings: &self.mappings,
                price: &self.price,
            },
            UpdateContext::new(&Exit::new()),
        )?;
        self.compute_aggregated(age, plugin, from)
    }

    fn compute_aggregated(&self, age: &AgeVecs, plugin: &mut Vecs, from: usize) -> Result<()> {
        let reader = self.history.reader(&self.spends, &self.creations)?;
        plugin.compute(
            Dependencies {
                history: &reader,
                from: Height::from(from),
                age,
                mappings: &self.mappings,
                price: &self.price,
            },
            UpdateContext::new(&Exit::new()),
        )
    }

    fn truncate(&mut self, from: usize) {
        self.spends.truncate(from).unwrap();
        self.creations.truncate(from).unwrap();
        self.price
            .spot
            .cents
            .height
            .truncate_if_needed_at(from)
            .unwrap();
        self.mappings
            .timestamp
            .monotonic
            .truncate_if_needed_at(from)
            .unwrap();
    }

    // Scan canonical origins independently of Age's accounting and the aggregate index.
    fn check(&self, plugin: &Vecs) {
        let reader = self.history.reader(&self.spends, &self.creations).unwrap();
        let prices = self.price.spot.cents.height.collect();
        let timestamps = self.mappings.timestamp.monotonic.collect();
        for h in 0..reader.len() {
            let state = reader.state_at(h + 1).unwrap();
            for &filter in AgeAggregateId::ALL {
                let mut supply = 0_u64;
                let mut count = 0_u64;
                let mut cap = 0_u128;
                let mut capitalized = 0_u128;
                let mut profit_supply = 0_u64;
                let mut sorted = Vec::new();
                for (origin, amount) in state.amounts().iter().enumerate() {
                    let age = AgeRangeId::from(Age::new(timestamps[h], timestamps[origin]));
                    if !filter.contains(age) {
                        continue;
                    }
                    let price = Cents::from(CentsCompact::from(prices[origin]));
                    supply += amount.sats;
                    count += amount.count;
                    cap += price.as_u128() * u128::from(amount.sats);
                    capitalized += price.as_u128().pow(2) * u128::from(amount.sats);
                    if price <= prices[h] {
                        profit_supply += amount.sats;
                    }
                    if amount.sats != 0 {
                        let mut represented = price.round_to_dollar(5).min(Cents::new(100_000_000));
                        if represented.inner() >= 10_000_000 {
                            represented = Cents::new(represented.inner() / 1000 * 1000);
                        }
                        sorted.push((
                            represented,
                            amount.sats,
                            price.as_u128() * u128::from(amount.sats),
                        ));
                    }
                }
                sorted.sort_by_key(|v| v.0);
                let m = filter.select(&plugin.cohorts);
                assert_eq!(
                    m.supply.total.sats.height.collect_one_at(h),
                    Some(Sats::new(supply)),
                    "supply {h} {filter:?}"
                );
                assert_eq!(
                    m.outputs
                        .unspent_count
                        .height
                        .collect_one_at(h)
                        .map(u64::from)
                        .unwrap(),
                    count,
                    "count {h} {filter:?}"
                );
                assert_eq!(
                    m.realized.price.cents.height.collect_one_at(h),
                    Some(Cents::new(
                        cap.checked_div(u128::from(supply)).unwrap_or_default() as u64
                    )),
                    "price {h} {filter:?}"
                );
                assert_eq!(
                    m.realized.capitalized_price.cents.height.collect_one_at(h),
                    Some(Cents::new(
                        capitalized.checked_div(cap).unwrap_or_default() as u64
                    )),
                    "capitalized price {h} {filter:?}"
                );
                assert_eq!(
                    m.supply.in_profit.sats.height.collect_one_at(h),
                    Some(Sats::new(profit_supply)),
                    "profit supply {h} {filter:?}"
                );
                assert_eq!(
                    m.supply.in_loss.sats.height.collect_one_at(h),
                    Some(Sats::new(supply - profit_supply))
                );
                let market = prices[h].as_u128() * u128::from(supply) / Sats::ONE_BTC_U128;
                let net = (prices[h].as_u128() * u128::from(supply)) as i128 - cap as i128;
                let expected_nupl = if market == 0 {
                    PartsPerMillionSigned32::ZERO
                } else {
                    PartsPerMillionSigned32::from(
                        (net / Sats::ONE_BTC_U128 as i128) as f64 / market as f64,
                    )
                };
                assert_eq!(
                    m.unrealized.nupl.ppm.height.collect_one_at(h),
                    Some(expected_nupl),
                    "NUPL {h} {filter:?}",
                );
                assert_eq!(
                    m.cost_basis.min.cents.height.collect_one_at(h),
                    Some(sorted.first().map_or(Cents::ZERO, |v| v.0))
                );
                assert_eq!(
                    m.cost_basis.max.cents.height.collect_one_at(h),
                    Some(sorted.last().map_or(Cents::ZERO, |v| v.0))
                );
                for ((&pct, coin), dollar) in PERCENTILES
                    .iter()
                    .zip(m.cost_basis.per_coin.iter())
                    .zip(m.cost_basis.per_dollar.iter())
                {
                    let target = |total: u128| (total * u128::from(pct) / 100).saturating_sub(1);
                    let quantile = |dollar: bool| {
                        let total = if dollar { cap } else { u128::from(supply) };
                        let mut cumulative = 0;
                        sorted
                            .iter()
                            .find_map(|(price, sats, cap)| {
                                cumulative += if dollar { *cap } else { u128::from(*sats) };
                                (cumulative > target(total)).then_some(*price)
                            })
                            .unwrap_or(Cents::ZERO)
                    };
                    assert_eq!(
                        coin.cents.height.collect_one_at(h),
                        Some(quantile(false)),
                        "coin percentile {h} {filter:?} {pct}"
                    );
                    assert_eq!(
                        dollar.cents.height.collect_one_at(h),
                        Some(quantile(true)),
                        "dollar percentile {h} {filter:?} {pct}"
                    );
                }
            }
        }
    }
}

fn amount(btc: u64, count: u64) -> Amount {
    Amount {
        sats: btc * 100_000_000,
        count,
    }
}

#[test]
fn uniform_cohorts_match_origin_scan_through_append_restart_reorg_and_repair() {
    thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let mut f = Fixture::new();
            f.append(0, 10_003, amount(3, 4), &[], 0);
            f.append(10, 20_049, amount(2, 2), &[], 0);
            f.append(119, 30_101, amount(1, 2), &[(2, amount(0, 1))], 0);
            f.append(120, 10_000, amount(1, 1), &[(0, amount(1, 1))], 0);
            f.publish(0);
            let (mut age, mut plugin) = f.import();
            f.compute(&mut age, &mut plugin, 0).unwrap();
            f.check(&plugin);
            let same_layout = |plugin: &Vecs| {
                let lengths: Vec<_> = plugin
                    .cohorts
                    .iter()
                    .map(|m| m.iter_any_visible().count())
                    .collect();
                assert!(lengths.iter().all(|n| *n == lengths[0]));
            };
            same_layout(&plugin);
            f.compute(&mut age, &mut plugin, 4).unwrap();
            f.append(149, 40_000, amount(1, 1), &[(1, amount(1, 1))], 0);
            f.append(150, 20_000, amount(1, 1), &[(0, amount(1, 1))], 0);
            f.append(180, 70_000_007, amount(1, 1), &[], 0);
            f.publish(4);
            f.compute(&mut age, &mut plugin, 4).unwrap();
            f.check(&plugin);
            let readonly = plugin.read_only_clone();
            assert_eq!(
                readonly.cohorts.all.realized.price.cents.height.collect(),
                plugin.cohorts.all.realized.price.cents.height.collect()
            );
            drop(readonly);
            drop((age, plugin));
            let (mut age, mut plugin) = f.import();
            f.append(400, 25_000, amount(1, 1), &[(1, amount(1, 1))], 0);
            f.publish(7);
            f.compute(&mut age, &mut plugin, 7).unwrap();
            f.check(&plugin);
            plugin
                .cohorts
                .all
                .columns
                .price
                .any_truncate_if_needed_at(3)
                .unwrap();
            f.compute(&mut age, &mut plugin, 8).unwrap();
            f.check(&plugin);
            // Repair a derived ratio without restoring or replaying the history index.
            let prices = plugin.live.as_ref().unwrap().prices.as_ptr();
            plugin
                .cohorts
                .all
                .relative
                .supply_dominance
                .ppm
                .height
                .any_truncate_if_needed_at(2)
                .unwrap();
            f.compute(&mut age, &mut plugin, 8).unwrap();
            assert_eq!(plugin.live.as_ref().unwrap().prices.as_ptr(), prices);
            assert_eq!(
                plugin
                    .cohorts
                    .all
                    .relative
                    .supply_dominance
                    .ppm
                    .height
                    .len(),
                8
            );
            f.truncate(5);
            f.append(149, 60_000, amount(2, 2), &[(0, amount(1, 1))], 1);
            f.append(150, 70_000, amount(1, 1), &[], 1);
            f.append(180, 30_000, amount(1, 1), &[], 1);
            f.publish(5);
            f.compute(&mut age, &mut plugin, 5).unwrap();
            f.check(&plugin);
            f.price.spot.cents.height = import_cached(&f.db, "spot_v2", Version::TWO).unwrap();
            for h in 0..8 {
                f.price.spot.cents.height.push(Cents::new(50_000 + h * 100));
            }
            f.price.spot.cents.height.write().unwrap();
            f.compute(&mut age, &mut plugin, 8).unwrap();
            f.check(&plugin);
            f.price.spot.cents.height.truncate_if_needed_at(7).unwrap();
            f.price.spot.cents.height.push(Cents::NAN);
            f.price.spot.cents.height.write().unwrap();
            assert!(f.compute_aggregated(&age, &mut plugin, 7).is_err());
            assert!(plugin.live.is_none());
            f.price.spot.cents.height.truncate_if_needed_at(7).unwrap();
            f.price.spot.cents.height.push(Cents::new(20_000));
            f.price.spot.cents.height.write().unwrap();
            f.compute(&mut age, &mut plugin, 7).unwrap();
            f.check(&plugin);
            drop((age, plugin));
            let (mut age, mut plugin) = f.import();
            f.compute(&mut age, &mut plugin, 0).unwrap();
            f.check(&plugin);
            same_layout(&plugin);
        })
        .unwrap()
        .join()
        .unwrap();
}
