use std::path::Path;

use brk_cohort::{
    AGE_RANGE_COUNT, AgeRange, AgeRangeId, AmountRange, ByEntry, ByEpoch, Class, CohortContext,
    Filter, OverAge, OverAmount, SpendableType, Term, UnderAge, UnderAmount,
};
use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::{Cents, CentsSquaredSats, Height, Sats, Version};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, AnyVec, BinaryTransform, CachedBoxedVec, ColumnId, ColumnarVec, Database,
    EagerVec, Exit, ImportableVec, PcoVec, ReadOnlyClone, ReadableVec, Rw, StorageMode,
    WritableVec,
};

use crate::{
    distribution::{
        AllChainCache, DynCohortVecs,
        metrics::{
            AgeRangeSupplySources, AllCohortMetrics, AllSupplyCache, BasicCohortMetrics,
            CohortMetricsBase, CoreCohortMetrics, ExtendedAdjustedCohortMetrics,
            ExtendedCohortMetrics, ImportConfig, MinimalCohortMetrics, ProfitabilityMetrics,
            RealizedFullAccum, TypeCohortMetrics,
        },
        state::UTXOCohortState,
    },
    indexes,
    internal::{
        CachedWindowStartVec, LazyColumnValuePerBlockCumulativeRolling, SatsToCents, Windows,
    },
    price,
};

use super::{fenwick::CostBasisFenwick, vecs::UTXOCohortVecs};

const VERSION: Version = Version::new(0);

/// All UTXO cohorts organized by filter type.
#[derive(Traversable)]
pub struct UTXOCohorts<M: StorageMode = Rw> {
    pub all: UTXOCohortVecs<AllCohortMetrics<M>>,
    pub sth: UTXOCohortVecs<ExtendedAdjustedCohortMetrics<M>>,
    pub lth: UTXOCohortVecs<ExtendedCohortMetrics<M>>,
    pub age_range: AgeRange<UTXOCohortVecs<BasicCohortMetrics<M>>>,
    pub under_age: UnderAge<UTXOCohortVecs<CoreCohortMetrics<M>>>,
    pub over_age: OverAge<UTXOCohortVecs<CoreCohortMetrics<M>>>,
    pub epoch: ByEpoch<UTXOCohortVecs<CoreCohortMetrics<M>>>,
    pub class: Class<UTXOCohortVecs<CoreCohortMetrics<M>>>,
    pub entry: ByEntry<UTXOCohortVecs<CoreCohortMetrics<M>>>,
    pub over_amount: OverAmount<UTXOCohortVecs<MinimalCohortMetrics<M>>>,
    pub amount_range: AmountRange<UTXOCohortVecs<MinimalCohortMetrics<M>>>,
    pub under_amount: UnderAmount<UTXOCohortVecs<MinimalCohortMetrics<M>>>,
    #[traversable(rename = "type")]
    pub type_: SpendableType<UTXOCohortVecs<TypeCohortMetrics<M>>>,
    pub profitability: ProfitabilityMetrics<M>,
    #[traversable(skip)]
    age_range_supply: AgeRangeSupplySources<M>,
    pub matured: AgeRange<LazyColumnValuePerBlockCumulativeRolling<AgeRangeId>>,
    pub cumulative_matured_sats: M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Sats>, AgeRangeId>>>,
    pub cumulative_matured_cents:
        M::Stored<EagerVec<ColumnarVec<PcoVec<Height, Cents>, AgeRangeId>>>,
    #[traversable(skip)]
    all_supply_cache: AllSupplyCache,
    #[traversable(skip)]
    pub(super) caches: UTXOCohortsTransientState,
}

/// In-memory state that does NOT survive rollback.
#[derive(Clone, Default)]
pub(crate) struct UTXOCohortsTransientState {
    pub(super) fenwick: CostBasisFenwick,
    /// Cached partition_point positions for tick_tock boundary searches.
    /// Avoids O(log n) binary search per boundary per block; scans forward
    /// from last known position (typically O(1) per boundary).
    pub(super) tick_tock_cached_positions: [usize; AGE_RANGE_COUNT - 1],
}

const MATURED_VERSION: Version = Version::new(4);
const AGE_RANGE_SUPPLY_VERSION: Version = Version::ONE;
const WRITE_INTERVAL: usize = 10_000;

impl UTXOCohorts<Rw> {
    /// Separate cohorts currently total 74:
    /// 23 age + 5 epoch + 18 class + 2 entry + 15 amount + 11 spendable type.
    /// Keep small headroom because this is only Vec allocation capacity.
    const SEPARATE_COHORT_CAPACITY: usize = 84;

    /// Import all UTXO cohorts from database.
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        states_path: &Path,
        cached_starts: &Windows<&CachedWindowStartVec>,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let v = version + VERSION;
        let prefix = CohortContext::Utxo.prefix();

        // Phase 1: Import the age-range supply source and its cached all-column view.
        let all_full_name = CohortContext::Utxo.full_name(&Filter::All, "");
        let all_cfg = ImportConfig {
            db,
            filter: &Filter::All,
            full_name: &all_full_name,
            version: v + Version::ONE,
            indexes,
            cached_starts,
            spot_price,
        };
        let age_range_supply =
            AgeRangeSupplySources::forced_import(db, prefix, v + AGE_RANGE_SUPPLY_VERSION)?;
        let (all_supply, all_supply_cache) = age_range_supply.all(&all_cfg);
        let all_chain_cache = AllChainCache::new(&all_supply_cache, spot_price);

        // Phase 2: Import separate (stateful) cohorts.
        let age_range = AgeRange::try_from_fn(|id| {
            let filter = id.filter().clone();
            let full_name = CohortContext::Utxo.full_name(&filter, id.name().id);
            let cfg = ImportConfig {
                db,
                filter: &filter,
                full_name: &full_name,
                version: v,
                indexes,
                cached_starts,
                spot_price,
            };
            let state = Some(Box::new(UTXOCohortState::new(states_path, &full_name)));
            let supply = age_range_supply.column(&cfg, id, &all_supply_cache);
            Ok::<_, brk_error::Error>(UTXOCohortVecs::new(
                state,
                BasicCohortMetrics::forced_import(&cfg, supply)?,
            ))
        })?;

        let core_separate =
            |f: Filter, name: &'static str| -> Result<UTXOCohortVecs<CoreCohortMetrics>> {
                let full_name = CohortContext::Utxo.full_name(&f, name);
                let cfg = ImportConfig {
                    db,
                    filter: &f,
                    full_name: &full_name,
                    version: v,
                    indexes,
                    cached_starts,
                    spot_price,
                };
                let state = Some(Box::new(UTXOCohortState::new(states_path, &full_name)));
                Ok(UTXOCohortVecs::new(
                    state,
                    CoreCohortMetrics::forced_import(&cfg, &all_supply_cache)?,
                ))
            };

        let epoch = ByEpoch::try_new(&core_separate)?;
        let class = Class::try_new(&core_separate)?;

        let entry = ByEntry::try_new(&core_separate)?;

        // Helper for separate cohorts with MinimalCohortMetrics + MinimalRealizedState
        let minimal_separate =
            |f: Filter, name: &'static str| -> Result<UTXOCohortVecs<MinimalCohortMetrics>> {
                let full_name = CohortContext::Utxo.full_name(&f, name);
                let cfg = ImportConfig {
                    db,
                    filter: &f,
                    full_name: &full_name,
                    version: v,
                    indexes,
                    cached_starts,
                    spot_price,
                };
                let state = Some(Box::new(UTXOCohortState::new(states_path, &full_name)));
                Ok(UTXOCohortVecs::new(
                    state,
                    MinimalCohortMetrics::forced_import(&cfg, &all_supply_cache)?,
                ))
            };

        let amount_range = AmountRange::try_new(&minimal_separate)?;

        let type_separate =
            |f: Filter, name: &'static str| -> Result<UTXOCohortVecs<TypeCohortMetrics>> {
                let full_name = CohortContext::Utxo.full_name(&f, name);
                let cfg = ImportConfig {
                    db,
                    filter: &f,
                    full_name: &full_name,
                    version: v,
                    indexes,
                    cached_starts,
                    spot_price,
                };
                let state = Some(Box::new(UTXOCohortState::new(states_path, &full_name)));
                Ok(UTXOCohortVecs::new(
                    state,
                    TypeCohortMetrics::forced_import(&cfg, &all_supply_cache)?,
                ))
            };

        let type_ = SpendableType::try_new(&type_separate)?;

        // Phase 3: Import "all" cohort with pre-imported supply.
        let all = UTXOCohortVecs::new(
            None,
            AllCohortMetrics::forced_import_with_supply(&all_cfg, all_supply, &all_chain_cache)?,
        );

        // Phase 3b: Import profitability metrics (derived from "all" during k-way merge).
        let profitability =
            ProfitabilityMetrics::forced_import(db, v, indexes, cached_starts, spot_price)?;

        // Phase 4: Import aggregate cohorts.

        // sth: ExtendedAdjustedCohortMetrics
        let sth = {
            let f = Filter::Term(Term::Sth);
            let full_name = CohortContext::Utxo.full_name(&f, "sth");
            let cfg = ImportConfig {
                db,
                filter: &f,
                full_name: &full_name,
                version: v,
                indexes,
                cached_starts,
                spot_price,
            };
            UTXOCohortVecs::new(
                None,
                ExtendedAdjustedCohortMetrics::forced_import(
                    &cfg,
                    age_range_supply.filtered(&cfg, &f, &all_supply_cache),
                    &all_chain_cache,
                )?,
            )
        };

        // lth: ExtendedCohortMetrics
        let lth = {
            let f = Filter::Term(Term::Lth);
            let full_name = CohortContext::Utxo.full_name(&f, "lth");
            let cfg = ImportConfig {
                db,
                filter: &f,
                full_name: &full_name,
                version: v,
                indexes,
                cached_starts,
                spot_price,
            };
            UTXOCohortVecs::new(
                None,
                ExtendedCohortMetrics::forced_import(
                    &cfg,
                    age_range_supply.filtered(&cfg, &f, &all_supply_cache),
                    &all_chain_cache,
                )?,
            )
        };

        // CoreCohortMetrics without state (no state, for aggregate cohorts)
        let core_no_state =
            |f: Filter, name: &'static str| -> Result<UTXOCohortVecs<CoreCohortMetrics>> {
                let full_name = CohortContext::Utxo.full_name(&f, name);
                let cfg = ImportConfig {
                    db,
                    filter: &f,
                    full_name: &full_name,
                    version: v,
                    indexes,
                    cached_starts,
                    spot_price,
                };
                let supply = age_range_supply.filtered(&cfg, &f, &all_supply_cache);
                Ok(UTXOCohortVecs::new(
                    None,
                    CoreCohortMetrics::forced_import_with_supply(&cfg, supply)?,
                ))
            };

        // under_age: CoreCohortMetrics (no state, aggregates from age_range)
        let under_age = UnderAge::try_new(&core_no_state)?;

        // over_age: CoreCohortMetrics (no state, aggregates from age_range)
        let over_age = OverAge::try_new(&core_no_state)?;

        let minimal_no_state =
            |f: Filter, name: &'static str| -> Result<UTXOCohortVecs<MinimalCohortMetrics>> {
                let full_name = CohortContext::Utxo.full_name(&f, name);
                let cfg = ImportConfig {
                    db,
                    filter: &f,
                    full_name: &full_name,
                    version: v,
                    indexes,
                    cached_starts,
                    spot_price,
                };
                Ok(UTXOCohortVecs::new(
                    None,
                    MinimalCohortMetrics::forced_import(&cfg, &all_supply_cache)?,
                ))
            };

        let under_amount = UnderAmount::try_new(&minimal_no_state)?;
        let over_amount = OverAmount::try_new(&minimal_no_state)?;

        let matured_version = v + MATURED_VERSION;
        let matured_sats =
            EagerVec::<ColumnarVec<PcoVec<Height, Sats>, AgeRangeId>>::forced_import(
                db,
                &format!("{prefix}_age_range_matured_supply_cumulative_sats"),
                matured_version,
            )?;
        let matured_cents =
            EagerVec::<ColumnarVec<PcoVec<Height, Cents>, AgeRangeId>>::forced_import(
                db,
                &format!("{prefix}_age_range_matured_supply_cumulative_cents"),
                matured_version,
            )?;
        let matured_sats_ref = matured_sats.read_only_clone();
        let matured_cents_ref = matured_cents.read_only_clone();
        let matured = AgeRangeId::series(CohortContext::Utxo, |column, name| {
            LazyColumnValuePerBlockCumulativeRolling::new(
                &format!("{name}_matured_supply"),
                matured_version,
                &matured_sats_ref,
                &matured_cents_ref,
                column,
                indexes,
                cached_starts,
            )
        });

        Ok(Self {
            all,
            sth,
            lth,
            epoch,
            class,
            entry,
            type_,
            under_age,
            over_age,
            age_range,
            amount_range,
            under_amount,
            over_amount,
            profitability,
            age_range_supply,
            matured,
            cumulative_matured_sats: matured_sats,
            cumulative_matured_cents: matured_cents,
            all_supply_cache,
            caches: UTXOCohortsTransientState::default(),
        })
    }

    /// Reset in-memory caches that become stale after rollback.
    pub(crate) fn reset_caches(&mut self) {
        self.all_supply_cache.clear();
        self.caches = UTXOCohortsTransientState::default();
    }

    pub(crate) fn all_supply_cache(&self) -> &AllSupplyCache {
        &self.all_supply_cache
    }

    /// Initialize the Fenwick tree from all age-range BTreeMaps.
    /// Call after state import when all pending maps have been drained.
    pub(crate) fn init_fenwick_if_needed(&mut self) {
        if self.caches.fenwick.is_initialized() {
            return;
        }
        let Self {
            sth,
            caches,
            age_range,
            ..
        } = self;
        caches.fenwick.compute_is_sth(&sth.metrics.filter);

        let maps: Vec<_> = AgeRangeId::ALL
            .iter()
            .filter_map(|&id| {
                let sub = id.select(age_range);
                let state = sub.state.as_ref()?;
                let map = state.cost_basis_map();
                if map.is_empty() {
                    return None;
                }
                Some((map, caches.fenwick.is_sth(id)))
            })
            .collect();
        caches.fenwick.bulk_init(maps.into_iter());
    }

    /// Apply pending deltas from all age-range cohorts to the Fenwick tree.
    /// Call after receive/send, before push_cohort_states.
    pub(crate) fn update_fenwick_from_pending(&mut self) {
        if !self.caches.fenwick.is_initialized() {
            return;
        }
        // Destructure to get separate borrows on caches and age_range
        let Self {
            caches, age_range, ..
        } = self;
        for &id in AgeRangeId::ALL {
            let sub = id.select(age_range);
            if let Some(state) = sub.state.as_ref() {
                let is_sth = caches.fenwick.is_sth(id);
                state.for_each_cost_basis_pending(|&price, delta| {
                    caches.fenwick.apply_delta(price, delta, is_sth);
                });
            }
        }
    }

    /// Push maturation sats to the matured vecs for the given height.
    #[inline(always)]
    pub(crate) fn push_maturation(&mut self, matured: &AgeRange<Sats>) {
        let mut cumulative = self
            .cumulative_matured_sats
            .collect_last()
            .unwrap_or_default();
        for column in AgeRangeId::ALL {
            *column.get_mut(&mut cumulative) += *column.select(matured);
        }
        self.cumulative_matured_sats.push(cumulative);
    }

    fn compute_matured_cents(
        &mut self,
        starting_height: Height,
        prices: &price::Vecs,
        exit: &Exit,
    ) -> Result<()> {
        let cumulative_sats = self.cumulative_matured_sats.read_only_clone();
        let mut previous_sats = None;

        self.cumulative_matured_cents.compute_transform2_batched(
            starting_height,
            &cumulative_sats,
            &prices.spot.cents.height,
            WRITE_INTERVAL,
            |(height, current_sats, price, target)| {
                let prior_sats = previous_sats.replace(current_sats).unwrap_or_else(|| {
                    height
                        .decremented()
                        .and_then(|height| cumulative_sats.collect_one(height))
                        .unwrap_or_default()
                });
                let mut current_cents = target.collect_last().unwrap_or_default();

                for column in AgeRangeId::ALL {
                    let block_sats = *column.get(&current_sats) - *column.get(&prior_sats);
                    *column.get_mut(&mut current_cents) += SatsToCents::apply(block_sats, price);
                }

                (height, current_cents)
            },
            exit,
        )?;
        Ok(())
    }

    pub(crate) fn par_iter_separate_mut(
        &mut self,
    ) -> impl ParallelIterator<Item = &mut dyn DynCohortVecs> {
        let Self {
            age_range,
            epoch,
            class,
            entry,
            amount_range,
            type_,
            ..
        } = self;
        age_range
            .par_iter_mut()
            .map(|x| x as &mut dyn DynCohortVecs)
            .chain(epoch.par_iter_mut().map(|x| x as &mut dyn DynCohortVecs))
            .chain(class.par_iter_mut().map(|x| x as &mut dyn DynCohortVecs))
            .chain(entry.par_iter_mut().map(|x| x as &mut dyn DynCohortVecs))
            .chain(
                amount_range
                    .par_iter_mut()
                    .map(|x| x as &mut dyn DynCohortVecs),
            )
            .chain(type_.par_iter_mut().map(|x| x as &mut dyn DynCohortVecs))
    }

    /// Sequential mutable iterator over all separate (stateful) cohorts.
    /// Use instead of `par_iter_separate_mut` when per-item work is trivial.
    pub(crate) fn iter_separate_mut(&mut self) -> impl Iterator<Item = &mut dyn DynCohortVecs> {
        let Self {
            age_range,
            epoch,
            class,
            entry,
            amount_range,
            type_,
            ..
        } = self;
        age_range
            .iter_mut()
            .map(|x| x as &mut dyn DynCohortVecs)
            .chain(epoch.iter_mut().map(|x| x as &mut dyn DynCohortVecs))
            .chain(class.iter_mut().map(|x| x as &mut dyn DynCohortVecs))
            .chain(entry.iter_mut().map(|x| x as &mut dyn DynCohortVecs))
            .chain(amount_range.iter_mut().map(|x| x as &mut dyn DynCohortVecs))
            .chain(type_.iter_mut().map(|x| x as &mut dyn DynCohortVecs))
    }

    /// Immutable iterator over all separate (stateful) cohorts.
    pub(crate) fn iter_separate(&self) -> impl Iterator<Item = &dyn DynCohortVecs> {
        self.age_range
            .iter()
            .map(|x| x as &dyn DynCohortVecs)
            .chain(self.epoch.iter().map(|x| x as &dyn DynCohortVecs))
            .chain(self.class.iter().map(|x| x as &dyn DynCohortVecs))
            .chain(self.entry.iter().map(|x| x as &dyn DynCohortVecs))
            .chain(self.amount_range.iter().map(|x| x as &dyn DynCohortVecs))
            .chain(self.type_.iter().map(|x| x as &dyn DynCohortVecs))
    }

    pub(crate) fn compute_overlapping_vecs(
        &mut self,
        starting_lengths: &Lengths,
        exit: &Exit,
    ) -> Result<()> {
        let Self {
            all,
            sth,
            lth,
            age_range,
            under_age,
            over_age,
            over_amount,
            amount_range,
            under_amount,
            ..
        } = self;

        let ar = &*age_range;
        let amr = &*amount_range;
        let si = starting_lengths;

        let tasks: Vec<Box<dyn FnOnce() -> Result<()> + Send + '_>> = vec![
            Box::new(|| {
                let sources = filter_sources_from(ar.iter(), None);
                all.metrics.compute_base_from_others(si, &sources, exit)
            }),
            Box::new(|| {
                let sources = filter_sources_from(ar.iter(), Some(sth.metrics.filter()));
                sth.metrics.compute_base_from_others(si, &sources, exit)
            }),
            Box::new(|| {
                let sources = filter_sources_from(ar.iter(), Some(lth.metrics.filter()));
                lth.metrics.compute_base_from_others(si, &sources, exit)
            }),
            Box::new(|| {
                over_age.par_iter_mut().try_for_each(|vecs| {
                    let sources = filter_sources_from(ar.iter(), Some(&vecs.metrics.filter));
                    vecs.metrics.compute_from_base_sources(si, &sources, exit)
                })
            }),
            Box::new(|| {
                under_age.par_iter_mut().try_for_each(|vecs| {
                    let sources = filter_sources_from(ar.iter(), Some(&vecs.metrics.filter));
                    vecs.metrics.compute_from_base_sources(si, &sources, exit)
                })
            }),
            Box::new(|| {
                over_amount
                    .par_iter_mut()
                    .chain(under_amount.par_iter_mut())
                    .try_for_each(|vecs| {
                        let sources =
                            filter_minimal_sources_from(amr.iter(), Some(&vecs.metrics.filter));
                        vecs.metrics.compute_from_sources(si, &sources, exit)
                    })
            }),
        ];

        tasks
            .into_par_iter()
            .map(|f| f())
            .collect::<Result<Vec<_>>>()?;

        Ok(())
    }

    /// First phase of post-processing: compute index transforms.
    pub(crate) fn compute_rest_part1(
        &mut self,
        prices: &price::Vecs,
        starting_lengths: &Lengths,
        exit: &Exit,
    ) -> Result<()> {
        // 1. Compute all metrics except net_sentiment (all cohorts via DynCohortVecs)
        {
            let mut all: Vec<&mut dyn DynCohortVecs> =
                Vec::with_capacity(Self::SEPARATE_COHORT_CAPACITY + 3);
            all.push(&mut self.all);
            all.push(&mut self.sth);
            all.push(&mut self.lth);
            all.extend(
                self.under_age
                    .iter_mut()
                    .map(|x| x as &mut dyn DynCohortVecs),
            );
            all.extend(
                self.over_age
                    .iter_mut()
                    .map(|x| x as &mut dyn DynCohortVecs),
            );
            all.extend(
                self.over_amount
                    .iter_mut()
                    .map(|x| x as &mut dyn DynCohortVecs),
            );
            all.extend(
                self.age_range
                    .iter_mut()
                    .map(|x| x as &mut dyn DynCohortVecs),
            );
            all.extend(self.epoch.iter_mut().map(|x| x as &mut dyn DynCohortVecs));
            all.extend(self.class.iter_mut().map(|x| x as &mut dyn DynCohortVecs));
            all.extend(self.entry.iter_mut().map(|x| x as &mut dyn DynCohortVecs));
            all.extend(
                self.amount_range
                    .iter_mut()
                    .map(|x| x as &mut dyn DynCohortVecs),
            );
            all.extend(
                self.under_amount
                    .iter_mut()
                    .map(|x| x as &mut dyn DynCohortVecs),
            );
            all.extend(self.type_.iter_mut().map(|x| x as &mut dyn DynCohortVecs));
            all.into_par_iter()
                .try_for_each(|v| v.compute_rest_part1(prices, starting_lengths, exit))?;
        }

        // Compute matured cumulative cents from the cumulative sats matrix × price.
        self.compute_matured_cents(starting_lengths.height, prices, exit)?;

        // Compute profitability supply cents and realized price
        self.profitability.compute(prices, starting_lengths, exit)?;

        Ok(())
    }

    /// Second phase of post-processing: compute relative metrics.
    pub(crate) fn compute_rest_part2(
        &mut self,
        prices: &price::Vecs,
        starting_lengths: &Lengths,
        exit: &Exit,
    ) -> Result<()> {
        // Get under_1h value sources for adjusted computation (cloned to avoid borrow conflicts).
        let under_1h_value_created = self
            .age_range
            .under_1h
            .metrics
            .activity
            .transfer_volume
            .block
            .cents
            .clone();
        let under_1h_value_destroyed = self
            .age_range
            .under_1h
            .metrics
            .realized
            .sopr
            .value_destroyed
            .cumulative
            .height
            .read_only_clone();

        // "all" cohort computed first.
        self.all.metrics.compute_rest_part2(
            prices,
            starting_lengths,
            &under_1h_value_created,
            &under_1h_value_destroyed,
            exit,
        )?;

        // Destructure to allow parallel mutable access to independent fields.
        let Self {
            sth,
            lth,
            age_range,
            under_age,
            over_age,
            over_amount,
            amount_range,
            under_amount,
            epoch,
            class,
            entry,
            type_,
            ..
        } = self;

        // All remaining groups run in parallel. Each closure owns an exclusive &mut
        // to its field and shares read-only references to common data.
        let vc = &under_1h_value_created;
        let vd = &under_1h_value_destroyed;
        let tasks: Vec<Box<dyn FnOnce() -> Result<()> + Send + '_>> = vec![
            Box::new(|| {
                sth.metrics
                    .compute_rest_part2(prices, starting_lengths, vc, vd, exit)
            }),
            Box::new(|| {
                lth.metrics
                    .compute_rest_part2(prices, starting_lengths, exit)
            }),
            Box::new(|| {
                age_range
                    .par_iter_mut()
                    .try_for_each(|v| v.metrics.compute_rest_part2(prices, starting_lengths, exit))
            }),
            Box::new(|| {
                under_age
                    .par_iter_mut()
                    .try_for_each(|v| v.metrics.compute_rest_part2(prices, starting_lengths, exit))
            }),
            Box::new(|| {
                over_age
                    .par_iter_mut()
                    .try_for_each(|v| v.metrics.compute_rest_part2(prices, starting_lengths, exit))
            }),
            Box::new(|| {
                over_amount
                    .par_iter_mut()
                    .try_for_each(|v| v.metrics.compute_rest_part2(prices, starting_lengths, exit))
            }),
            Box::new(|| {
                epoch
                    .par_iter_mut()
                    .try_for_each(|v| v.metrics.compute_rest_part2(prices, starting_lengths, exit))
            }),
            Box::new(|| {
                class
                    .par_iter_mut()
                    .try_for_each(|v| v.metrics.compute_rest_part2(prices, starting_lengths, exit))
            }),
            Box::new(|| {
                entry
                    .par_iter_mut()
                    .try_for_each(|v| v.metrics.compute_rest_part2(prices, starting_lengths, exit))
            }),
            Box::new(|| {
                amount_range
                    .par_iter_mut()
                    .try_for_each(|v| v.metrics.compute_rest_part2(prices, starting_lengths, exit))
            }),
            Box::new(|| {
                under_amount
                    .par_iter_mut()
                    .try_for_each(|v| v.metrics.compute_rest_part2(prices, starting_lengths, exit))
            }),
            Box::new(|| {
                type_
                    .par_iter_mut()
                    .try_for_each(|v| v.metrics.compute_rest_part2(prices, starting_lengths, exit))
            }),
        ];

        tasks
            .into_par_iter()
            .map(|f| f())
            .collect::<Result<Vec<_>>>()?;

        Ok(())
    }

    /// Returns a parallel iterator over all vecs for parallel writing.
    pub(crate) fn par_iter_vecs_mut(
        &mut self,
    ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = Vec::with_capacity(2048);
        vecs.extend(self.all.metrics.collect_all_vecs_mut());
        vecs.extend(self.sth.metrics.collect_all_vecs_mut());
        vecs.extend(self.lth.metrics.collect_all_vecs_mut());
        for v in self.age_range.iter_mut() {
            vecs.extend(v.metrics.collect_all_vecs_mut());
        }
        for v in self.under_age.iter_mut() {
            vecs.extend(v.metrics.collect_all_vecs_mut());
        }
        for v in self.over_age.iter_mut() {
            vecs.extend(v.metrics.collect_all_vecs_mut());
        }
        for v in self.over_amount.iter_mut() {
            vecs.extend(v.metrics.collect_all_vecs_mut());
        }
        for v in self.epoch.iter_mut() {
            vecs.extend(v.metrics.collect_all_vecs_mut());
        }
        for v in self.class.iter_mut() {
            vecs.extend(v.metrics.collect_all_vecs_mut());
        }
        for v in self.entry.iter_mut() {
            vecs.extend(v.metrics.collect_all_vecs_mut());
        }
        for v in self.amount_range.iter_mut() {
            vecs.extend(v.metrics.collect_all_vecs_mut());
        }
        for v in self.under_amount.iter_mut() {
            vecs.extend(v.metrics.collect_all_vecs_mut());
        }
        for v in self.type_.iter_mut() {
            vecs.extend(v.metrics.collect_all_vecs_mut());
        }
        vecs.extend(self.profitability.collect_all_vecs_mut());
        vecs.extend(self.age_range_supply.stored_vecs_mut());
        vecs.push(&mut self.cumulative_matured_sats);
        vecs.push(&mut self.cumulative_matured_cents);
        vecs.into_par_iter()
    }

    /// Commit all states to disk (separate from vec writes for parallelization).
    pub(crate) fn commit_all_states(&mut self, height: Height, cleanup: bool) -> Result<()> {
        self.par_iter_separate_mut()
            .try_for_each(|v| v.write_state(height, cleanup))
    }

    pub(crate) fn min_stateful_len(&self) -> Height {
        self.iter_separate()
            .map(|v| Height::from(v.min_stateful_len()))
            .chain(std::iter::once(Height::from(
                self.cumulative_matured_sats.len(),
            )))
            .min()
            .unwrap_or_default()
            .min(Height::from(self.age_range_supply.len()))
            .min(Height::from(self.profitability.min_stateful_len()))
            .min(Height::from(self.all.min_stateful_len()))
            .min(Height::from(self.sth.min_stateful_len()))
            .min(Height::from(self.lth.min_stateful_len()))
    }

    /// Import state for all separate cohorts at or before given height.
    /// Returns true if all imports succeeded and returned the expected height.
    pub(crate) fn import_separate_states(&mut self, height: Height) -> bool {
        self.par_iter_separate_mut()
            .map(|v| v.import_state(height).unwrap_or_default())
            .all(|h| h == height)
    }

    /// Reset state heights for all separate cohorts.
    pub(crate) fn reset_separate_state_heights(&mut self) {
        self.iter_separate_mut()
            .for_each(|v| v.reset_state_starting_height());
    }

    /// Reset cost_basis_data for all separate cohorts (called during fresh start).
    pub(crate) fn reset_separate_cost_basis_data(&mut self) -> Result<()> {
        self.iter_separate_mut()
            .try_for_each(|v| v.reset_cost_basis_data_if_needed())
    }

    /// Validate computed versions for all cohorts.
    pub(crate) fn validate_computed_versions(&mut self, base_version: Version) -> Result<()> {
        // Validate separate cohorts
        self.iter_separate_mut()
            .try_for_each(|v| v.validate_computed_versions(base_version))?;

        // Validate aggregate cohorts
        self.all.metrics.validate_computed_versions(base_version)?;
        self.sth.metrics.validate_computed_versions(base_version)?;
        self.lth.metrics.validate_computed_versions(base_version)?;
        for v in self.over_age.iter_mut() {
            v.metrics.validate_computed_versions(base_version)?;
        }
        for v in self.under_age.iter_mut() {
            v.metrics.validate_computed_versions(base_version)?;
        }
        Ok(())
    }

    /// Aggregate RealizedFull fields from age_range states and push to all/sth/lth.
    /// Called during the block loop after separate cohorts' push_state but before reset.
    pub(crate) fn push_overlapping(&mut self, height_price: Cents) -> Cents {
        let Self {
            all,
            sth,
            lth,
            age_range,
            age_range_supply,
            ..
        } = self;

        let sth_filter = &sth.metrics.filter;

        let mut all_acc = RealizedFullAccum::default();
        let mut sth_acc = RealizedFullAccum::default();
        let mut lth_acc = RealizedFullAccum::default();

        let mut all_ccap = (0u128, 0u128);
        let mut sth_ccap = (0u128, 0u128);
        let mut lth_ccap = (0u128, 0u128);

        let mut total_supply = AgeRangeId::from_fn(|_| Sats::ZERO);
        let mut supply_in_profit = AgeRangeId::from_fn(|_| Sats::ZERO);
        let mut supply_in_loss = AgeRangeId::from_fn(|_| Sats::ZERO);

        for &id in AgeRangeId::ALL {
            let ar = id.select_mut(age_range);
            if let Some(state) = ar.state.as_mut() {
                all_acc.add(&state.realized);

                let u = state.compute_unrealized_state(height_price);
                *id.get_mut(&mut total_supply) = state.supply.value;
                *id.get_mut(&mut supply_in_profit) = u.supply_in_profit;
                *id.get_mut(&mut supply_in_loss) = u.supply_in_loss;
                all_ccap.0 += u.capitalized_cap_in_profit_raw;
                all_ccap.1 += u.capitalized_cap_in_loss_raw;

                if sth_filter.includes(&ar.metrics.filter) {
                    sth_acc.add(&state.realized);
                    sth_ccap.0 += u.capitalized_cap_in_profit_raw;
                    sth_ccap.1 += u.capitalized_cap_in_loss_raw;
                } else {
                    lth_acc.add(&state.realized);
                    lth_ccap.0 += u.capitalized_cap_in_profit_raw;
                    lth_ccap.1 += u.capitalized_cap_in_loss_raw;
                }
            }
        }

        age_range_supply.push(total_supply, supply_in_profit, supply_in_loss);

        let all_capitalized_price = all.metrics.realized.push_accum(&all_acc);
        sth.metrics.realized.push_accum(&sth_acc);
        lth.metrics.realized.push_accum(&lth_acc);

        all.metrics
            .unrealized
            .capitalized_cap_in_profit_raw
            .push(CentsSquaredSats::new(all_ccap.0));
        all.metrics
            .unrealized
            .capitalized_cap_in_loss_raw
            .push(CentsSquaredSats::new(all_ccap.1));
        sth.metrics
            .unrealized
            .capitalized_cap_in_profit_raw
            .push(CentsSquaredSats::new(sth_ccap.0));
        sth.metrics
            .unrealized
            .capitalized_cap_in_loss_raw
            .push(CentsSquaredSats::new(sth_ccap.1));
        lth.metrics
            .unrealized
            .capitalized_cap_in_profit_raw
            .push(CentsSquaredSats::new(lth_ccap.0));
        lth.metrics
            .unrealized
            .capitalized_cap_in_loss_raw
            .push(CentsSquaredSats::new(lth_ccap.1));

        all_capitalized_price
    }
}

/// Filter source cohorts by an optional filter.
/// If filter is None, returns all sources (used for "all" aggregate).
fn filter_sources_from<'a, M: CohortMetricsBase + 'a>(
    sources: impl Iterator<Item = &'a UTXOCohortVecs<M>>,
    filter: Option<&Filter>,
) -> Vec<&'a M> {
    match filter {
        Some(f) => sources
            .filter(|v| f.includes(v.metrics.filter()))
            .map(|v| &v.metrics)
            .collect(),
        None => sources.map(|v| &v.metrics).collect(),
    }
}

/// Filter MinimalCohortMetrics source cohorts by an optional filter.
fn filter_minimal_sources_from<'a>(
    sources: impl Iterator<Item = &'a UTXOCohortVecs<MinimalCohortMetrics>>,
    filter: Option<&Filter>,
) -> Vec<&'a MinimalCohortMetrics> {
    match filter {
        Some(f) => sources
            .filter(|v| f.includes(&v.metrics.filter))
            .map(|v| &v.metrics)
            .collect(),
        None => sources.map(|v| &v.metrics).collect(),
    }
}
