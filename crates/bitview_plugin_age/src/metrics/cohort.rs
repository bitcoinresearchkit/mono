use bitview_cohort::CohortId;
use bitview_collections::Windows;
use bitview_distribution::state::RealizedOps;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use super::{
    activity::ActivityVecs, outputs::OutputsVecs, realized::RealizedVecs, supply::SupplyVecs,
    unrealized::UnrealizedVecs,
};
use crate::state::UTXOCohortState;

/// One creation cohort: an age range, a halving epoch or a creation year.
#[derive(Traversable)]
pub struct CohortVecs<M: StorageMode = Rw> {
    pub supply: SupplyVecs<M>,
    pub outputs: OutputsVecs<M>,
    pub activity: ActivityVecs<M>,
    pub realized: RealizedVecs<M>,
    pub unrealized: UnrealizedVecs<M>,
}

impl CohortVecs {
    pub(crate) fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        Ok(Self {
            supply: SupplyVecs::import(
                db,
                cohort,
                version,
                mappings,
                window_starts,
                spot_price,
                all_supply,
            )?,
            outputs: OutputsVecs::import(db, cohort, version, mappings, window_starts)?,
            activity: ActivityVecs::import(db, cohort, version, mappings, window_starts)?,
            realized: RealizedVecs::import(db, cohort, version, mappings, window_starts)?,
            unrealized: UnrealizedVecs::import(db, cohort, version, mappings)?,
        })
    }

    /// Writes the block's values, with the cohort's profitability at `price`.
    #[inline(always)]
    pub(crate) fn push<R: RealizedOps>(&mut self, state: &mut UTXOCohortState<R>, price: Cents) {
        let profitability = state.compute_unrealized_state(price);
        self.supply.push(state.supply_value(), &profitability);
        self.unrealized.push(&profitability);
        self.outputs.push(state.output_counts());
        self.activity
            .push(state.transfer_volume(), state.core_activity(), price);
        self.realized.push(&state.realized_block_data());
    }

    pub(crate) fn stored_vecs_mut(&mut self) -> impl Iterator<Item = &mut dyn AnyStoredVec> {
        self.supply
            .stored_vecs_mut()
            .into_iter()
            .chain(self.outputs.stored_vecs_mut())
            .chain(self.activity.stored_vecs_mut())
            .chain(self.realized.stored_vecs_mut())
            .chain(self.unrealized.stored_vecs_mut())
    }
}
