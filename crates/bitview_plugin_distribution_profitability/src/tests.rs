use std::{sync::Once, thread};

use bitview_cohort::{
    AgeAggregateId, AgeRangeId, ProfitabilityRangeId, compute_profitability_boundaries,
};
use bitview_collections::Windows;
use bitview_plugin::{ComputePlugin, ImportContext, UpdateContext};
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_vecs::{CachedSeries, LazyWindowStartVec, import_cached};
use brk_error::Result;
use brk_exit::Exit;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::{
    Age, Cents, CentsCompact, CentsSats, Height, PartsPerMillionSigned32, Sats, Timestamp, Version,
};
use statedb::{Amount, Creations, History, Spends};
use tempfile::{TempDir, tempdir};
use vecdb::{
    AnyStoredVec, Budgeted, Database, ReadOnlyClone, ReadableCloneableVec, ReadableVec, WritableVec,
};

use crate::{Dependencies, Vecs};

struct Fixture {
    directory: TempDir,
    _indexer: Indexer,
    db: Database,
    mappings: Mappings,
    prices: CachedSeries<Height, Cents>,
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
        let db = Database::open(&directory.path().join("sources")).unwrap();
        let path = directory.path().join("history");
        Self {
            prices: import_cached(&db, "spot", Version::ONE).unwrap(),
            spends: Spends::open(&path).unwrap(),
            creations: Creations::open(&path).unwrap(),
            history: History::open(&path).unwrap(),
            directory,
            mappings,
            _indexer: indexer,
            db,
        }
    }

    fn import(&self) -> Vecs {
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
        Vecs::import(
            ImportContext::new(self.directory.path()),
            &self.mappings,
            &windows,
            &self.prices.read_only_boxed_clone(),
        )
        .unwrap()
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
        hash[..8].copy_from_slice(&(self.creations.len() as u64).to_le_bytes());
        self.spends.push(hash, removed.iter().copied()).unwrap();
        self.creations.push(hash, created, None).unwrap();
        self.prices.push(if price == u64::MAX {
            Cents::NAN
        } else {
            Cents::new(price)
        });
        self.mappings
            .timestamp
            .monotonic
            .push(Timestamp::new(1_231_006_505 + day * 86400));
    }

    fn publish(&mut self, from: usize) {
        self.spends.commit().unwrap();
        self.creations.commit().unwrap();
        self.history
            .advance(
                from,
                self.creations.len(),
                &self.spends,
                &self.creations,
                |_, _| Ok(()),
            )
            .unwrap();
        self.prices.write().unwrap();
        self.mappings.timestamp.monotonic.write().unwrap();
    }

    fn compute(&self, plugin: &mut Vecs, from: usize) -> Result<()> {
        let reader = self.history.reader(&self.spends, &self.creations)?;
        plugin.compute(
            Dependencies {
                history: &reader,
                from: Height::from(from),
                prices: &self.prices.read_only_boxed_clone(),
                timestamps: &self.mappings.timestamp.monotonic.read_only_boxed_clone(),
            },
            UpdateContext::new(&Exit::new()),
        )
    }

    fn truncate(&mut self, from: usize) {
        self.spends.truncate(from).unwrap();
        self.creations.truncate(from).unwrap();
        self.prices.truncate_if_needed_at(from).unwrap();
        self.mappings
            .timestamp
            .monotonic
            .truncate_if_needed_at(from)
            .unwrap();
    }

    // Independent origin scan: no incremental price index or cached cutoff positions.
    fn check(&self, plugin: &Vecs) {
        let reader = self.history.reader(&self.spends, &self.creations).unwrap();
        let prices = self.prices.collect();
        let timestamps = self.mappings.timestamp.monotonic.collect();
        for h in 0..reader.len() {
            let state = reader.state_at(h + 1).unwrap();
            let boundaries = compute_profitability_boundaries(prices[h]);
            let mut supply = [[0_u64; 7]; 25];
            let mut cap = [[0_u128; 7]; 25];
            for (origin, amount) in state.amounts().iter().enumerate() {
                let price = Cents::from(CentsCompact::from(prices[origin]));
                let represented = price.round_to_dollar(5).min(Cents::new(100_000_000));
                let band = boundaries.partition_point(|b| {
                    represented >= b.round_to_dollar(5).min(Cents::new(100_000_000))
                });
                let age = AgeRangeId::from(Age::new(timestamps[h], timestamps[origin]));
                for &filter in AgeAggregateId::ALL {
                    if filter.contains(age) {
                        supply[band][filter.index()] += amount.sats;
                        cap[band][filter.index()] += price.as_u128() * u128::from(amount.sats);
                    }
                }
            }
            for &band in ProfitabilityRangeId::ALL {
                let caps = cap[band.index()].map(|c| CentsSats::new(c).to_cents_rounded());
                for &filter in AgeAggregateId::ALL {
                    assert_eq!(
                        filter
                            .select(band.select(&plugin.metrics.supply))
                            .sats
                            .height
                            .collect_one_at(h),
                        Some(Sats::new(supply[band.index()][filter.index()])),
                        "supply at {h}, {band:?}, {filter:?}"
                    );
                    assert_eq!(
                        filter
                            .select(band.select(&plugin.metrics.realized_cap))
                            .cents
                            .height
                            .collect_one_at(h),
                        Some(caps[filter.index()]),
                        "cap at {h}, {band:?}, {filter:?}"
                    );
                    let sats = supply[band.index()][filter.index()];
                    let pnl = |i: usize| {
                        let market = CentsSats::from_price_sats(
                            prices[h],
                            Sats::new(supply[band.index()][i]),
                        )
                        .to_cents_rounded();
                        if band.is_profit() {
                            market.saturating_sub(caps[i])
                        } else {
                            caps[i].saturating_sub(market)
                        }
                    };
                    let expected_pnl = pnl(filter.index());
                    assert_eq!(
                        filter
                            .select(band.select(&plugin.metrics.unrealized_pnl))
                            .cents
                            .height
                            .collect_one_at(h),
                        Some(expected_pnl)
                    );
                    let expected_nupl = if sats == 0 || prices[h] == Cents::ZERO {
                        PartsPerMillionSigned32::ZERO
                    } else {
                        let realized_price =
                            caps[filter.index()].as_u128() * Sats::ONE_BTC_U128 / u128::from(sats);
                        PartsPerMillionSigned32::from(
                            (prices[h].as_u128() as f64 - realized_price as f64)
                                / prices[h].as_u128() as f64,
                        )
                    };
                    assert_eq!(
                        filter
                            .select(band.select(&plugin.metrics.nupl))
                            .ppm
                            .height
                            .collect_one_at(h),
                        Some(expected_nupl)
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
fn complete_update_handles_boundaries_restart_reorg_and_failed_update() {
    thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(|| {
            let mut f = Fixture::new();
            f.append(0, 10_003, amount(3, 3), &[], 0);
            f.append(10, 20_049, amount(2, 2), &[], 0);
            f.append(119, 30_101, amount(1, 2), &[(2, amount(0, 1))], 0);
            f.append(120, 10_000, amount(1, 1), &[(0, amount(1, 1))], 0);
            f.publish(0);
            let mut plugin = f.import();
            f.compute(&mut plugin, 0).unwrap();
            f.check(&plugin);
            f.compute(&mut plugin, 4).unwrap(); // no-op retains the writer-owned state
            f.append(149, 40_000, amount(1, 1), &[(1, amount(1, 1))], 0);
            f.append(150, 20_000, amount(1, 1), &[(0, amount(1, 1))], 0);
            f.append(180, 50_000, amount(1, 1), &[], 0);
            f.publish(4);
            f.compute(&mut plugin, 4).unwrap();
            f.check(&plugin);
            drop(plugin);
            let mut plugin = f.import();
            f.append(400, 25_000, amount(1, 1), &[(1, amount(1, 1))], 0);
            f.publish(7);
            f.compute(&mut plugin, 7).unwrap(); // restore and jump across several cutoffs
            f.check(&plugin);
            plugin
                .metrics
                .stored_vecs_mut()
                .next()
                .unwrap()
                .any_truncate_if_needed_at(3)
                .unwrap();
            f.compute(&mut plugin, 8).unwrap(); // shortest persisted source repairs every metric
            f.check(&plugin);
            f.truncate(5);
            f.append(149, 60_000, amount(2, 2), &[(0, amount(1, 1))], 1);
            f.append(150, 70_000, amount(1, 1), &[], 1);
            f.append(180, 30_000, amount(1, 1), &[], 1);
            f.publish(5);
            f.compute(&mut plugin, 5).unwrap();
            f.check(&plugin);
            let readonly = plugin.read_only_clone();
            assert_eq!(
                readonly
                    .metrics
                    .supply
                    ._0pct_to_10pct_in_profit
                    .all
                    .sats
                    .height
                    .collect(),
                plugin
                    .metrics
                    .supply
                    ._0pct_to_10pct_in_profit
                    .all
                    .sats
                    .height
                    .collect()
            );
            drop(readonly);
            f.prices = import_cached(&f.db, "spot_v2", Version::TWO).unwrap();
            for h in 0..8 {
                f.prices.push(Cents::new(50_000 + h * 100));
            }
            f.prices.write().unwrap();
            f.compute(&mut plugin, 8).unwrap(); // source version invalidates both output and cache
            f.check(&plugin);
            f.append(181, u64::MAX, amount(1, 1), &[], 1);
            f.publish(8);
            assert!(f.compute(&mut plugin, 8).is_err());
            assert!(plugin.live.is_none());
            f.prices.truncate_if_needed_at(8).unwrap();
            f.prices.push(Cents::new(20_000));
            f.prices.write().unwrap();
            f.compute(&mut plugin, 8).unwrap();
            f.check(&plugin);
            drop(plugin);
            let mut plugin = f.import();
            f.compute(&mut plugin, 0).unwrap();
            f.check(&plugin);
        })
        .unwrap()
        .join()
        .unwrap();
}
