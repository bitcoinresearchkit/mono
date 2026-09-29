use std::thread;

use bitview_cohort::{
    AgeRange, AgeRangeId, ByEntry, ByEpoch, Class, Term, UTXOAggregate, UTXOAllAndSth,
    UTXOCoreValues, UTXOGroupsWithoutAmountOrType,
};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Lengths, Sats, StoredU64, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, ReadOnlyClone, ReadableBoxedVec, Rw, StorageMode};

use crate::{
    metrics::{
        ActivityVecs, CostBasisVecs, OutputsVecs, ProfitabilityVecs, RealizedAggregateSources,
        RealizedAggregateState, RealizedVecs, RelativeSource, RelativeVecs, Sopr24hInput,
        SupplyVecs, UnrealizedVecs,
    },
    state::{UTXOStates, UnrealizedState},
};

const VERSION: Version = Version::new(0);
const IMPORT_STACK_SIZE: usize = 8 * 1024 * 1024;

#[cfg(test)]
#[path = "cohorts_tests.rs"]
mod tests;

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
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
        all_count: &ReadableBoxedVec<Height, StoredU64>,
    ) -> Result<Self> {
        let v = version + VERSION;

        // Supply must exist before either branch can build its shared views.
        let supply =
            SupplyVecs::forced_import(db, v, mappings, window_starts, spot_price, all_supply)?;
        let all_market_cap = supply.total.all_market_cap();

        // These branches are independent once the supply sources exist.
        let ((realized, unrealized, relative), (outputs, activity)) =
            thread::scope(|scope| -> Result<_> {
                let outputs_activity = thread::Builder::new()
                    .stack_size(IMPORT_STACK_SIZE)
                    .spawn_scoped(scope, || -> Result<_> {
                        let outputs =
                            OutputsVecs::forced_import(db, v, mappings, window_starts, all_count)?;
                        let activity = ActivityVecs::forced_import(db, v, mappings, window_starts)?;
                        Ok((outputs, activity))
                    })?;

                let realized = RealizedVecs::forced_import(
                    db,
                    v,
                    mappings,
                    window_starts,
                    spot_price,
                    all_market_cap,
                )?;
                let unrealized =
                    UnrealizedVecs::forced_import(db, v, mappings, &realized.price.cohorts)?;
                let relative_sources = UTXOAggregate::from_fn(|id| {
                    let cohort_id = id.cohort();
                    RelativeSource {
                        supply: supply.sources(cohort_id).expect("aggregate supply sources"),
                        unrealized: unrealized
                            .sources(cohort_id)
                            .expect("aggregate unrealized sources"),
                        unrealized_aggregate: unrealized
                            .aggregate_sources(cohort_id)
                            .expect("aggregate unrealized sources"),
                        realized: realized
                            .sources(cohort_id)
                            .expect("aggregate realized sources"),
                        nupl: unrealized
                            .nupl
                            .get(cohort_id)
                            .expect("aggregate NUPL source"),
                    }
                });
                let relative = RelativeVecs::forced_import(
                    db,
                    v,
                    mappings,
                    all_market_cap,
                    &relative_sources,
                )?;
                Ok((
                    (realized, unrealized, relative),
                    outputs_activity.join().unwrap()?,
                ))
            })?;
        let cost_basis = CostBasisVecs::forced_import(db, v, mappings)?;
        let profitability =
            ProfitabilityVecs::forced_import(db, v, mappings, window_starts, spot_price)?;

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

    pub fn all_supply(&self) -> &ReadableBoxedVec<Height, Sats> {
        self.supply.total.all_supply()
    }

    pub fn all_market_cap(&self) -> &ReadableBoxedVec<Height, Cents> {
        self.supply.total.all_market_cap()
    }

    fn sopr_24h_inputs(&self) -> UTXOGroupsWithoutAmountOrType<Sopr24hInput> {
        UTXOGroupsWithoutAmountOrType::new(|cohort_id| {
            Sopr24hInput::new(
                self.activity
                    .transfer_volume
                    .cohorts
                    .get(cohort_id)
                    .expect("SOPR transfer-volume cohort"),
                self.realized
                    .value_destroyed
                    .cohorts
                    .get(cohort_id)
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
            ..
        } = states;

        let total = UTXOCoreValues {
            age_range: AgeRange::from_fn(|id| id.select(age_range).supply_value()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).supply_value()),
            class: Class::from_fn(|id| id.select(class).supply_value()),
            entry: ByEntry::from_fn(|id| id.select(entry).supply_value()),
        };
        let profitability = UTXOCoreValues {
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
        };

        let mut aggregates = UTXOAggregate::default();
        for id in AgeRangeId::ALL {
            let state = id.select(&profitability.age_range);
            aggregates.all += state;
            if id.term() == Term::Sth {
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
            ..
        } = states;

        let cohort_values = UTXOCoreValues {
            age_range: AgeRange::from_fn(|id| id.select(age_range).output_counts()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).output_counts()),
            class: Class::from_fn(|id| id.select(class).output_counts()),
            entry: ByEntry::from_fn(|id| id.select(entry).output_counts()),
        };
        let unspent_count = cohort_values.map(|counts| counts.0);
        let spent_count = cohort_values.map(|counts| counts.1);
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
            ..
        } = states;

        let transfer_volume = UTXOCoreValues {
            age_range: AgeRange::from_fn(|id| id.select(age_range).transfer_volume()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).transfer_volume()),
            class: Class::from_fn(|id| id.select(class).transfer_volume()),
            entry: ByEntry::from_fn(|id| id.select(entry).transfer_volume()),
        };
        let core = UTXOCoreValues {
            age_range: AgeRange::from_fn(|id| id.select(age_range).core_activity()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).core_activity()),
            class: Class::from_fn(|id| id.select(class).core_activity()),
            entry: ByEntry::from_fn(|id| id.select(entry).core_activity()),
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
            ..
        } = states;

        realized.cap_raw.push_age(&AgeRange::from_fn(|id| {
            id.select(age_range).realized.cap_raw()
        }));
        realized
            .capitalized_cap_raw
            .push_age(&AgeRange::from_fn(|id| {
                id.select(age_range).realized.capitalized_cap_raw()
            }));

        let cohort_values = UTXOCoreValues {
            age_range: AgeRange::from_fn(|id| id.select(age_range).realized_block_data()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).realized_block_data()),
            class: Class::from_fn(|id| id.select(class).realized_block_data()),
            entry: ByEntry::from_fn(|id| id.select(entry).realized_block_data()),
        };
        realized.push(&cohort_values);
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
            .under_1h
            .cumulative
            .cents
            .height
            .read_only_clone();

        let sopr_inputs = self.sopr_24h_inputs();
        let realized_sources = UTXOAggregate::from_fn(|id| {
            let cohort_id = id.cohort();
            RealizedAggregateSources {
                activity: self
                    .activity
                    .sources(cohort_id)
                    .expect("aggregate activity sources"),
                realized: self
                    .realized
                    .sources(cohort_id)
                    .expect("aggregate realized sources"),
            }
        });
        let adjusted_sources = UTXOAllAndSth {
            all: &realized_sources.all,
            sth: &realized_sources.sth,
        };
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
            let cohort_id = id.cohort();
            RelativeSource {
                supply: self
                    .supply
                    .sources(cohort_id)
                    .expect("aggregate supply sources"),
                unrealized: self
                    .unrealized
                    .sources(cohort_id)
                    .expect("aggregate unrealized sources"),
                unrealized_aggregate: self
                    .unrealized
                    .aggregate_sources(cohort_id)
                    .expect("aggregate unrealized sources"),
                realized: self
                    .realized
                    .sources(cohort_id)
                    .expect("aggregate realized sources"),
                nupl: self
                    .unrealized
                    .nupl
                    .get(cohort_id)
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
    pub fn push_aggregate(
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

        let mut accumulated = UTXOAggregate::<RealizedAggregateState>::default();

        for id in AgeRangeId::ALL {
            let state = id.select(&states.age_range);
            accumulated.all.add(&state.realized);

            if id.term() == Term::Sth {
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
