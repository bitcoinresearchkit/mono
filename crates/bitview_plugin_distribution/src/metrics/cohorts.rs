use std::thread;

use bitview_cohort::{
    AgeRange, AgeRangeId, AmountRange, ByEntry, ByEpoch, Class, Filter, SpendableType, Term,
    UTXO_AGGREGATE_FILTERS, UTXOAggregate, UTXOAllAndSth, UTXOGroupsWithoutAmountOrType,
};
use bitview_plugin_indexer::Lengths;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, StoredU64, Version};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, CachedBoxedVec, ColumnId, Database, Exit, ReadOnlyClone, Rw, StorageMode,
};

use crate::{
    AllChainSources,
    metrics::{
        ActivityVecs, AdjustedSoprComputeSource, CostBasisVecs, OutputsVecs, ProfitabilityVecs,
        RealizedAggregateSources, RealizedAggregateState, RealizedVecs, RelativeSource,
        RelativeVecs, Sopr24hInput, SupplyVecs, UTXORows, UnrealizedVecs,
    },
    state::{AddrCohortState, RealizedOps, UTXOStates, UnrealizedState},
};
use bitview_compute::{CachedWindowStartVec, Windows};

const VERSION: Version = Version::new(0);
const IMPORT_STACK_SIZE: usize = 8 * 1024 * 1024;

/// Distribution metrics organized by metric, with cohorts at the leaves.
#[derive(Traversable)]
pub struct CohortMetrics<M: StorageMode = Rw> {
    pub supply: Box<SupplyVecs<M>>,
    pub outputs: Box<OutputsVecs<M>>,
    pub activity: Box<ActivityVecs<M>>,
    pub realized: Box<RealizedVecs<M>>,
    pub unrealized: Box<UnrealizedVecs<M>>,
    pub cost_basis: Box<CostBasisVecs<M>>,
    pub relative: Box<RelativeVecs<M>>,
    pub profitability: Box<ProfitabilityVecs<M>>,
}

impl CohortMetrics<Rw> {
    /// Import all cohort metrics from the database.
    pub fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let v = version + VERSION;

        // Supply must exist before either branch can build its shared views.
        let supply = Box::new(SupplyVecs::forced_import(
            db,
            v,
            mappings,
            cached_starts,
            spot_price,
        )?);
        let all_chain_sources =
            AllChainSources::new(supply.total.all_supply(), supply.total.all_market_cap());

        // These branches are independent once the supply sources exist.
        let ((realized, unrealized, relative), (outputs, activity)) =
            thread::scope(|scope| -> Result<_> {
                let outputs_activity = thread::Builder::new()
                    .stack_size(IMPORT_STACK_SIZE)
                    .spawn_scoped(scope, || -> Result<_> {
                        let outputs =
                            Box::new(OutputsVecs::forced_import(db, v, mappings, cached_starts)?);
                        let activity =
                            Box::new(ActivityVecs::forced_import(db, v, mappings, cached_starts)?);
                        Ok((outputs, activity))
                    })?;

                let realized = Box::new(RealizedVecs::forced_import(
                    db,
                    v,
                    mappings,
                    cached_starts,
                    spot_price,
                    &all_chain_sources,
                )?);
                let unrealized = Box::new(UnrealizedVecs::forced_import(
                    db,
                    v,
                    mappings,
                    &realized.price.cohorts,
                )?);
                let relative_sources = UTXOAggregate::from_fn(|id| {
                    let filter = id.select(&UTXO_AGGREGATE_FILTERS);
                    RelativeSource {
                        supply: supply.sources(filter).expect("aggregate supply sources"),
                        unrealized: unrealized
                            .sources(filter)
                            .expect("aggregate unrealized sources"),
                        unrealized_aggregate: unrealized
                            .aggregate_sources(filter)
                            .expect("aggregate unrealized sources"),
                        realized: realized
                            .sources(filter)
                            .expect("aggregate realized sources"),
                        nupl: unrealized.nupl.get(filter).expect("aggregate NUPL source"),
                    }
                });
                let relative = Box::new(RelativeVecs::forced_import(
                    db,
                    v,
                    mappings,
                    &all_chain_sources,
                    &relative_sources,
                )?);
                Ok((
                    (realized, unrealized, relative),
                    outputs_activity.join().unwrap()?,
                ))
            })?;
        let cost_basis = Box::new(CostBasisVecs::forced_import(db, v, mappings)?);
        let profitability = Box::new(ProfitabilityVecs::forced_import(
            db,
            v,
            mappings,
            cached_starts,
            spot_price,
        )?);

        Ok(Self {
            supply,
            outputs,
            activity,
            realized,
            unrealized,
            cost_basis,
            relative,
            profitability,
        })
    }

    pub fn invalidate_caches(&self) {
        self.supply.total.all_supply().invalidate();
        self.supply.total.all_market_cap().invalidate();
    }

    pub fn all_supply(&self) -> &CachedBoxedVec<Height, Sats> {
        self.supply.total.all_supply()
    }

    pub fn all_market_cap(&self) -> &CachedBoxedVec<Height, Cents> {
        self.supply.total.all_market_cap()
    }

    fn sopr_24h_inputs(&self) -> UTXOGroupsWithoutAmountOrType<Sopr24hInput> {
        UTXOGroupsWithoutAmountOrType::new(|filter, _| {
            Sopr24hInput::new(
                self.activity
                    .transfer_volume
                    .cohorts
                    .get(&filter)
                    .expect("SOPR transfer-volume cohort"),
                self.realized
                    .value_destroyed
                    .cohorts
                    .get(&filter)
                    .expect("SOPR value-destroyed cohort"),
            )
        })
    }

    #[inline(always)]
    pub fn push_supply_and_unrealized(
        &mut self,
        states: &mut UTXOStates,
        height_price: Cents,
    ) -> UTXOAggregate<UnrealizedState> {
        let Self {
            supply, unrealized, ..
        } = self;
        let UTXOStates {
            age_range,
            epoch,
            class,
            entry,
            amount_range,
            type_,
            ..
        } = states;

        let total = UTXORows {
            age_range: AgeRange::from_fn(|id| id.select(age_range).supply_value()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).supply_value()),
            class: Class::from_fn(|id| id.select(class).supply_value()),
            entry: ByEntry::from_fn(|id| id.select(entry).supply_value()),
            amount_range: AmountRange::from_fn(|id| id.select(amount_range).supply_value()),
            type_: SpendableType::from_fn(|id| id.select(type_).supply_value()),
        };
        let profitability = UTXORows {
            age_range: AgeRange::from_fn(|id| {
                id.select_mut(age_range)
                    .compute_unrealized_state(height_price)
            }),
            epoch: ByEpoch::from_fn(|id| {
                id.select_mut(epoch).compute_unrealized_state(height_price)
            }),
            class: Class::from_fn(|id| id.select_mut(class).compute_unrealized_state(height_price)),
            entry: ByEntry::from_fn(|id| {
                id.select_mut(entry).compute_unrealized_state(height_price)
            }),
            amount_range: AmountRange::default(),
            type_: SpendableType::from_fn(|id| {
                id.select_mut(type_).compute_unrealized_state(height_price)
            }),
        };

        let mut aggregates = UTXOAggregate::default();
        let sth_filter = Filter::Term(Term::Sth);
        for id in AgeRangeId::ALL {
            let state = id.select(&profitability.age_range);
            aggregates.all += state;
            if sth_filter.includes(id.filter()) {
                aggregates.sth += state;
            } else {
                aggregates.lth += state;
            }
        }

        supply.push(total, &profitability);
        unrealized.push(&profitability, height_price, &aggregates);
        aggregates
    }

    #[inline(always)]
    pub fn push_outputs(&mut self, states: &UTXOStates) {
        let outputs = &mut self.outputs;
        let UTXOStates {
            age_range,
            epoch,
            class,
            entry,
            amount_range,
            type_,
            ..
        } = states;

        let rows = UTXORows {
            age_range: AgeRange::from_fn(|id| id.select(age_range).output_counts()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).output_counts()),
            class: Class::from_fn(|id| id.select(class).output_counts()),
            entry: ByEntry::from_fn(|id| id.select(entry).output_counts()),
            amount_range: AmountRange::from_fn(|id| id.select(amount_range).output_counts()),
            type_: SpendableType::from_fn(|id| id.select(type_).output_counts()),
        };
        let unspent_count = rows.map(|counts| counts.0);
        let spent_count = rows.map(|counts| counts.1);
        outputs.push(unspent_count, spent_count);
    }

    #[inline(always)]
    pub fn push_activity(&mut self, states: &UTXOStates, height_price: Cents) {
        let activity = &mut self.activity;
        let UTXOStates {
            age_range,
            epoch,
            class,
            entry,
            amount_range,
            type_,
            ..
        } = states;

        let transfer_volume = UTXORows {
            age_range: AgeRange::from_fn(|id| id.select(age_range).transfer_volume()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).transfer_volume()),
            class: Class::from_fn(|id| id.select(class).transfer_volume()),
            entry: ByEntry::from_fn(|id| id.select(entry).transfer_volume()),
            amount_range: AmountRange::from_fn(|id| id.select(amount_range).transfer_volume()),
            type_: SpendableType::from_fn(|id| id.select(type_).transfer_volume()),
        };
        let core = UTXORows {
            age_range: AgeRange::from_fn(|id| id.select(age_range).core_activity()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).core_activity()),
            class: Class::from_fn(|id| id.select(class).core_activity()),
            entry: ByEntry::from_fn(|id| id.select(entry).core_activity()),
            amount_range: AmountRange::default(),
            type_: SpendableType::default(),
        };
        let coindays_destroyed = core.map(|values| values.0);
        let transfer_volume_in_profit = core.map(|values| values.1);
        let transfer_volume_in_loss = core.map(|values| values.2);

        activity.push(
            height_price,
            transfer_volume,
            coindays_destroyed,
            transfer_volume_in_profit,
            transfer_volume_in_loss,
        );
    }

    #[inline(always)]
    pub fn push_realized(&mut self, states: &UTXOStates) {
        let realized = &mut self.realized;
        let UTXOStates {
            age_range,
            epoch,
            class,
            entry,
            amount_range,
            type_,
            ..
        } = states;

        let rows = UTXORows {
            age_range: AgeRange::from_fn(|id| id.select(age_range).realized_block_data()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).realized_block_data()),
            class: Class::from_fn(|id| id.select(class).realized_block_data()),
            entry: ByEntry::from_fn(|id| id.select(entry).realized_block_data()),
            amount_range: AmountRange::from_fn(|id| id.select(amount_range).realized_block_data()),
            type_: SpendableType::from_fn(|id| id.select(type_).realized_block_data()),
        };
        realized.push(&rows);
    }

    #[inline(always)]
    pub fn push_addr_balance(
        &mut self,
        states: &AmountRange<AddrCohortState>,
        height_price: Cents,
    ) {
        let supply = AmountRange::from_fn(|amount| amount.select(states).inner.supply.value);
        let output_count = AmountRange::from_fn(|amount| {
            StoredU64::from(amount.select(states).inner.supply.utxo_count)
        });
        let transfer_volume = AmountRange::from_fn(|amount| amount.select(states).inner.sent);
        let realized_cap =
            AmountRange::from_fn(|amount| amount.select(states).inner.realized.cap());
        let realized_profit =
            AmountRange::from_fn(|amount| amount.select(states).inner.realized.profit());
        let realized_loss =
            AmountRange::from_fn(|amount| amount.select(states).inner.realized.loss());

        self.supply.total.push_addr_balance(supply);
        self.outputs.push_addr_balance(output_count);
        self.activity
            .push_addr_balance(height_price, &transfer_volume);
        self.realized
            .push_addr_balance(realized_cap, &realized_profit, &realized_loss);
    }

    /// First phase of post-processing: compute index transforms.
    pub fn compute_rest_part1(&mut self, starting_lengths: &Lengths, exit: &Exit) -> Result<()> {
        self.activity
            .compute_dormancy(starting_lengths.height, exit)?;

        Ok(())
    }

    /// Second phase of post-processing: compute derived ratios and relative metrics.
    pub fn compute_rest_part2(&mut self, starting_lengths: &Lengths, exit: &Exit) -> Result<()> {
        // Get under_1h value sources for adjusted computation (cloned to avoid borrow conflicts).
        let under_1h_value_created_cumulative = self
            .activity
            .transfer_volume
            .cohorts
            .age
            .range
            .under_1h
            .cumulative
            .cents
            .height
            .read_only_clone();
        let under_1h_value_destroyed_cumulative = self
            .realized
            .value_destroyed
            .cohorts
            .age
            .range
            .under_1h
            .cumulative
            .cents
            .height
            .read_only_clone();

        let sopr_inputs = self.sopr_24h_inputs();
        let adjusted_sources = UTXOAllAndSth {
            all: AdjustedSoprComputeSource {
                activity: self
                    .activity
                    .sources(&Filter::All)
                    .expect("all activity sources"),
                realized: self
                    .realized
                    .sources(&Filter::All)
                    .expect("all realized sources"),
            },
            sth: AdjustedSoprComputeSource {
                activity: self
                    .activity
                    .sources(&Filter::Term(Term::Sth))
                    .expect("STH activity sources"),
                realized: self
                    .realized
                    .sources(&Filter::Term(Term::Sth))
                    .expect("STH realized sources"),
            },
        };
        let realized_sources = UTXOAggregate::from_fn(|id| {
            let filter = id.select(&UTXO_AGGREGATE_FILTERS);
            RealizedAggregateSources {
                activity: self
                    .activity
                    .sources(filter)
                    .expect("aggregate activity sources"),
                realized: self
                    .realized
                    .sources(filter)
                    .expect("aggregate realized sources"),
            }
        });
        let Self {
            realized: realized_vecs,
            relative,
            ..
        } = self;

        realized_vecs.compute_adjusted_sopr(
            starting_lengths.height,
            &adjusted_sources,
            &under_1h_value_created_cumulative,
            &under_1h_value_destroyed_cumulative,
            exit,
        )?;
        realized_vecs.compute_sopr(starting_lengths.height, &sopr_inputs, exit)?;

        realized_vecs.compute_aggregate_metrics(
            starting_lengths.height,
            &realized_sources,
            exit,
        )?;

        let relative_sources = UTXOAggregate::from_fn(|id| {
            let filter = id.select(&UTXO_AGGREGATE_FILTERS);
            RelativeSource {
                supply: self
                    .supply
                    .sources(filter)
                    .expect("aggregate supply sources"),
                unrealized: self
                    .unrealized
                    .sources(filter)
                    .expect("aggregate unrealized sources"),
                unrealized_aggregate: self
                    .unrealized
                    .aggregate_sources(filter)
                    .expect("aggregate unrealized sources"),
                realized: self
                    .realized
                    .sources(filter)
                    .expect("aggregate realized sources"),
                nupl: self
                    .unrealized
                    .nupl
                    .get(filter)
                    .expect("aggregate NUPL source"),
            }
        });
        relative.compute(starting_lengths.height, &relative_sources, exit)?;

        Ok(())
    }

    /// Returns a parallel iterator over all vecs for parallel writing.
    pub fn par_iter_vecs_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = Vec::with_capacity(128);
        vecs.extend(self.supply.collect_vecs_mut());
        vecs.extend(self.outputs.collect_vecs_mut());
        vecs.extend(self.activity.collect_vecs_mut());
        vecs.extend(self.realized.collect_vecs_mut());
        vecs.extend(self.unrealized.collect_vecs_mut());
        vecs.extend(self.cost_basis.collect_vecs_mut());
        vecs.extend(self.relative.collect_vecs_mut());
        vecs.extend(self.profitability.collect_all_vecs_mut());
        vecs.into_par_iter()
    }

    /// Minimum complete length across values produced by the block loop.
    /// Post-processing outputs must not participate in recovery.
    pub fn min_resume_len(&self) -> Height {
        Height::from(self.supply.min_resume_len())
            .min(Height::from(self.outputs.min_resume_len()))
            .min(Height::from(self.activity.min_resume_len()))
            .min(Height::from(self.realized.min_resume_len()))
            .min(Height::from(self.unrealized.min_resume_len()))
            .min(Height::from(self.cost_basis.min_resume_len()))
            .min(Height::from(self.profitability.min_resume_len()))
    }

    /// Validate computed versions for all cohorts.
    pub fn validate_computed_versions(&mut self, base_version: Version) -> Result<()> {
        self.cost_basis.validate_computed_versions(base_version)
    }

    /// Aggregate realized fields from age-range states and push all/STH/LTH.
    /// Called during the block loop after separate cohorts' push_state but before reset.
    pub fn push_overlapping(
        &mut self,
        states: &UTXOStates,
        height_price: Cents,
        unrealized_states: &UTXOAggregate<UnrealizedState>,
    ) -> Cents {
        let Self {
            realized,
            cost_basis,
            ..
        } = self;

        let sth_filter = Filter::Term(Term::Sth);

        let mut accumulated = UTXOAggregate::<RealizedAggregateState>::default();

        for id in AgeRangeId::ALL {
            let state = id.select(&states.age_range);
            accumulated.all.add(&state.realized);

            if sth_filter.includes(id.filter()) {
                accumulated.sth.add(&state.realized);
            } else {
                accumulated.lth.add(&state.realized);
            }
        }

        let all_capitalized_price = realized.push_aggregate(&accumulated);

        cost_basis.push_prices(height_price, unrealized_states);

        all_capitalized_price
    }
}
