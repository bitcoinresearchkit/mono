use std::{collections::BTreeSet, sync::Once, thread};

use bitview_collections::Windows;
use bitview_plugin::{ComputePlugin, ImportContext, UpdateContext};
use bitview_plugin_indexer::Indexer;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyWindowStartVec, import_cached};
use brk_exit::Exit;
use brk_reader::Reader;
use brk_rpc::{Auth, Client};
use brk_types::{Cents, Height, Sats, Timestamp, Version};
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
    _db: Database,
    mappings: Mappings,
    prices: CachedSeries<Height, Cents>,
    anchors: CachedSeries<Height, Cents>,
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
        let db = Database::open(&directory.path().join("sources")).unwrap();
        let history_path = directory.path().join("history");
        Self {
            prices: import_cached(&db, "spot", Version::ONE).unwrap(),
            anchors: import_cached(&db, "anchor", Version::ONE).unwrap(),
            supply: import_cached(&db, "global_supply", Version::ONE).unwrap(),
            spends: Spends::open(&history_path).unwrap(),
            creations: Creations::open(&history_path).unwrap(),
            history: History::open(&history_path).unwrap(),
            directory,
            mappings,
            _indexer: indexer,
            _db: db,
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
            &self.supply.read_only_boxed_clone(),
        )
        .unwrap()
    }

    fn append(
        &mut self,
        h: usize,
        price: u64,
        anchor: u64,
        created: Amount,
        removed: &[(u32, Amount)],
        fork: u8,
    ) {
        let mut hash = [fork; 32];
        hash[..8].copy_from_slice(&(h as u64).to_le_bytes());
        self.spends.push(hash, removed.iter().copied()).unwrap();
        self.creations.push(hash, created, None).unwrap();
        self.prices.push(Cents::new(price));
        self.anchors.push(Cents::new(anchor));
        self.mappings
            .timestamp
            .monotonic
            .push(Timestamp::new(1_231_006_505 + h as u32 * 600));
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
        let reader = self.history.reader(&self.spends, &self.creations).unwrap();
        self.supply.truncate_if_needed_at(from).unwrap();
        for h in from..reader.len() {
            self.supply
                .push(Sats::new(reader.state_at(h + 1).unwrap().total().sats));
        }
        self.supply.write().unwrap();
        self.prices.write().unwrap();
        self.anchors.write().unwrap();
        self.mappings.timestamp.monotonic.write().unwrap();
    }

    fn compute(&self, plugin: &mut Vecs, from: usize) {
        let reader = self.history.reader(&self.spends, &self.creations).unwrap();
        plugin
            .compute(
                Dependencies {
                    history: &reader,
                    from: Height::from(from),
                    prices: &self.prices.read_only_boxed_clone(),
                    timestamps: &self.mappings.timestamp.monotonic.read_only_boxed_clone(),
                    capitalized_price: &self.anchors.read_only_boxed_clone(),
                },
                UpdateContext::new(&Exit::new()),
            )
            .unwrap();
    }

    fn truncate(&mut self, from: usize) {
        self.spends.truncate(from).unwrap();
        self.creations.truncate(from).unwrap();
        self.prices.truncate_if_needed_at(from).unwrap();
        self.anchors.truncate_if_needed_at(from).unwrap();
        self.mappings
            .timestamp
            .monotonic
            .truncate_if_needed_at(from)
            .unwrap();
    }
}

fn amount(btc: u64, count: u64) -> Amount {
    Amount {
        sats: btc * 100_000_000,
        count,
    }
}

#[test]
fn canonical_history_survives_append_restart_reorg_and_partial_output() {
    thread::Builder::new()
        .stack_size(8 * 1024 * 1024)
        .spawn(check_lifecycle)
        .unwrap()
        .join()
        .unwrap();
}

fn check_lifecycle() {
    let mut f = Fixture::new();
    f.append(0, 100, 200, amount(1, 2), &[], 0);
    f.append(
        1,
        300,
        200,
        amount(2, 2),
        &[(
            0,
            Amount {
                sats: 50_000_000,
                count: 1,
            },
        )],
        0,
    );
    // Equality uses the previous anchor; the current anchor deliberately differs.
    // The zero-value output is created and spent in this same block.
    f.append(
        2,
        200,
        250,
        amount(1, 2),
        &[(1, amount(1, 1)), (2, amount(0, 1))],
        0,
    );
    f.publish(0);
    let mut plugin = f.import();
    f.compute(&mut plugin, 0);
    assert_eq!(
        plugin
            .cohorts
            .discount
            .supply
            .base
            .total
            .sats
            .height
            .collect(),
        [100_000_000, 50_000_000, 150_000_000].map(Sats::new)
    );
    assert_eq!(
        plugin.cohorts.premium.realized.cap.cents.height.collect(),
        [0, 600, 300].map(Cents::new)
    );
    assert_eq!(
        plugin
            .cohorts
            .premium
            .realized
            .loss
            .block
            .cents
            .collect_one_at(2),
        Some(Cents::new(100))
    );
    assert_eq!(
        u64::from(
            plugin
                .cohorts
                .discount
                .outputs
                .spent_count
                .block
                .collect_one_at(2)
                .unwrap()
        ),
        1
    );
    f.compute(&mut plugin, 3); // no-op
    f.append(
        3,
        400,
        250,
        amount(1, 1),
        &[
            (
                0,
                Amount {
                    sats: 50_000_000,
                    count: 1,
                },
            ),
            (1, amount(1, 1)),
        ],
        0,
    );
    f.publish(3);
    f.compute(&mut plugin, 3); // resident append
    drop(plugin);
    let mut plugin = f.import();
    f.append(
        4,
        100,
        500,
        amount(1, 1),
        &[(2, amount(1, 1)), (3, amount(1, 1))],
        0,
    );
    f.publish(4);
    f.compute(&mut plugin, 4); // restore canonical prefix after restart
    assert_eq!(
        plugin.cohorts.discount.realized.cap.cents.height.collect(),
        [100, 50, 250, 200, 100].map(Cents::new)
    );
    assert_eq!(
        plugin.cohorts.premium.realized.cap.cents.height.collect(),
        [0, 600, 300, 400, 0].map(Cents::new)
    );
    assert_eq!(
        plugin
            .cohorts
            .discount
            .realized
            .profit
            .cumulative
            .cents
            .height
            .collect_one_at(4),
        Some(Cents::new(250))
    );
    assert_eq!(
        plugin
            .cohorts
            .premium
            .realized
            .loss
            .cumulative
            .cents
            .height
            .collect_one_at(4),
        Some(Cents::new(400))
    );
    let before = plugin.cohorts.discount.realized.cap.cents.height.collect();
    let truncated = plugin
        .cohorts
        .discount
        .stored_vecs_mut()
        .into_iter()
        .find(|v| v.name().contains("unrealized_profit"))
        .unwrap();
    truncated.any_truncate_if_needed_at(3).unwrap();
    f.compute(&mut plugin, 5); // shortest persisted source controls recovery
    assert_eq!(
        plugin.cohorts.discount.realized.cap.cents.height.collect(),
        before
    );
    assert_eq!(
        plugin
            .cohorts
            .premium
            .realized
            .loss
            .cumulative
            .cents
            .height
            .collect_one_at(4),
        Some(Cents::new(400))
    );
    f.truncate(3);
    f.append(3, 200, 150, amount(1, 1), &[(1, amount(1, 1))], 1);
    f.append(4, 300, 500, amount(1, 1), &[(2, amount(1, 1))], 1);
    f.publish(3);
    f.compute(&mut plugin, 3); // same-length replacement changes birth membership
    assert_eq!(
        plugin.cohorts.discount.realized.cap.cents.height.collect(),
        [100, 50, 250, 450, 250].map(Cents::new)
    );
    assert_eq!(
        plugin.cohorts.premium.realized.cap.cents.height.collect(),
        [0, 600, 300, 0, 300].map(Cents::new)
    );
    let warm = plugin.cohorts.discount.realized.cap.cents.height.collect();
    let readonly = plugin.read_only_clone();
    assert_eq!(
        readonly
            .cohorts
            .discount
            .realized
            .cap
            .cents
            .height
            .collect(),
        warm
    );
    drop((readonly, plugin));
    let mut plugin = f.import();
    f.compute(&mut plugin, 0); // full replay matches resident/reconstructed updates
    assert_eq!(
        plugin.cohorts.discount.realized.cap.cents.height.collect(),
        warm
    );
    assert_eq!(
        plugin.cohorts.premium.realized.cap.cents.height.collect(),
        [0, 600, 300, 0, 300].map(Cents::new)
    );
    let names = plugin
        .iter_any_visible()
        .map(|v| v.name().to_owned())
        .collect::<BTreeSet<_>>();
    assert_eq!(names.len(), 630);
    assert!(names.contains("veteran_supply_sats"));
    assert!(names.contains("rookie_sopr_24h"));
    // Derived repair does not require replaying intact accounting sources.
    plugin
        .cohorts
        .premium
        .realized
        .sopr
        .height
        .truncate_if_needed_at(4)
        .unwrap();
    f.compute(&mut plugin, 5);
    assert_eq!(plugin.cohorts.premium.realized.sopr.height.len(), 5);
    // A changed anchor version invalidates both output history and resident membership.
    let old_version = f.anchors.read_only_boxed_clone().version();
    f.anchors = import_cached(&f._db, "anchor_v2", Version::TWO).unwrap();
    assert_ne!(f.anchors.read_only_boxed_clone().version(), old_version);
    for _ in 0..5 {
        f.anchors.push(Cents::ZERO);
    }
    f.anchors.write().unwrap();
    f.compute(&mut plugin, 5);
    assert_eq!(
        plugin
            .cohorts
            .discount
            .supply
            .base
            .total
            .sats
            .height
            .collect(),
        f.supply.collect()
    );
    assert_eq!(
        plugin
            .cohorts
            .premium
            .supply
            .base
            .total
            .sats
            .height
            .collect(),
        vec![Sats::ZERO; 5]
    );
}
