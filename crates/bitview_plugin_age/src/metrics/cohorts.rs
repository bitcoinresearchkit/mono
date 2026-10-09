use std::thread;

use bitview_cohort::{AgeRange, ByEpoch, Class, CreationCohorts};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, CentsSats, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use crate::{
    metrics::{ActivityVecs, OutputsVecs, RealizedVecs, SupplyVecs, UnrealizedVecs},
    state::UTXOStates,
};

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
}

impl CohortMetrics<Rw> {
    /// Import all cohort metrics from the database.
    pub fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let v = version + VERSION;

        // Supply must exist before either branch can build its shared views.
        let supply = SupplyVecs::import(db, v, mappings, window_starts, spot_price, all_supply)?;

        // These branches are independent once the supply sources exist.
        let ((realized, unrealized), (outputs, activity)) = thread::scope(|scope| -> Result<_> {
            let outputs_activity = thread::Builder::new()
                .stack_size(IMPORT_STACK_SIZE)
                .spawn_scoped(scope, || -> Result<_> {
                    let outputs = OutputsVecs::import(db, v, mappings, window_starts)?;
                    let activity = ActivityVecs::import(db, v, mappings, window_starts)?;
                    Ok((outputs, activity))
                })?;

            let realized = RealizedVecs::import(db, v, mappings, window_starts)?;
            let unrealized = UnrealizedVecs::import(db, v, mappings)?;
            Ok(((realized, unrealized), outputs_activity.join().unwrap()?))
        })?;

        Ok(Self {
            supply,
            outputs,
            activity,
            realized,
            unrealized,
        })
    }

    pub fn all_supply(&self) -> &ReadableBoxedVec<Height, Sats> {
        self.supply.total.all_supply()
    }

    #[inline(always)]
    pub fn push_supply_and_unrealized(&mut self, states: &mut UTXOStates, height_price: Cents) {
        let Self {
            supply, unrealized, ..
        } = self;
        let UTXOStates {
            age_range,
            epoch,
            class,
            ..
        } = states;

        let total = CreationCohorts {
            age: AgeRange::from_fn(|id| id.select(age_range).supply_value()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).supply_value()),
            class: Class::from_fn(|id| id.select(class).supply_value()),
        };
        let profitability = CreationCohorts {
            age: AgeRange::from_fn(|id| {
                id.select_mut(age_range)
                    .compute_unrealized_state(height_price)
            }),
            epoch: ByEpoch::from_fn(|id| {
                id.select_mut(epoch).compute_unrealized_state(height_price)
            }),
            class: Class::from_fn(|id| id.select_mut(class).compute_unrealized_state(height_price)),
        };

        supply.push(total, &profitability);
        unrealized.push(&profitability);
    }

    #[inline(always)]
    pub fn push_outputs(&mut self, states: &UTXOStates) {
        let outputs = &mut self.outputs;
        let UTXOStates {
            age_range,
            epoch,
            class,
            ..
        } = states;

        let cohort_values = CreationCohorts {
            age: AgeRange::from_fn(|id| id.select(age_range).output_counts()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).output_counts()),
            class: Class::from_fn(|id| id.select(class).output_counts()),
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
            ..
        } = states;

        let transfer_volume = CreationCohorts {
            age: AgeRange::from_fn(|id| id.select(age_range).transfer_volume()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).transfer_volume()),
            class: Class::from_fn(|id| id.select(class).transfer_volume()),
        };
        let core = CreationCohorts {
            age: AgeRange::from_fn(|id| id.select(age_range).core_activity()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).core_activity()),
            class: Class::from_fn(|id| id.select(class).core_activity()),
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
            ..
        } = states;

        realized.peak_regret_raw.push_age(&AgeRange::from_fn(|id| {
            CentsSats::new(id.select(age_range).realized.peak_regret_raw())
        }));
        realized.cap_raw.push_age(&AgeRange::from_fn(|id| {
            id.select(age_range).realized.cap_raw()
        }));
        realized.push_prices(&AgeRange::from_fn(|id| {
            let state = id.select(age_range);
            state
                .realized
                .cap_raw()
                .realized_price(state.supply_value())
        }));
        realized
            .capitalized_cap_raw
            .push_age(&AgeRange::from_fn(|id| {
                id.select(age_range).realized.capitalized_cap_raw()
            }));

        let cohort_values = CreationCohorts {
            age: AgeRange::from_fn(|id| id.select(age_range).realized_block_data()),
            epoch: ByEpoch::from_fn(|id| id.select(epoch).realized_block_data()),
            class: Class::from_fn(|id| id.select(class).realized_block_data()),
        };
        realized.push(&cohort_values);
    }

    /// Every vec the block loop writes.
    pub fn collect_vecs_mut(&mut self) -> Vec<&mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = Vec::with_capacity(128);
        vecs.extend(self.supply.collect_vecs_mut());
        vecs.extend(self.outputs.collect_vecs_mut());
        vecs.extend(self.activity.collect_vecs_mut());
        vecs.extend(self.realized.collect_vecs_mut());
        vecs.extend(self.unrealized.collect_vecs_mut());
        vecs
    }
}
