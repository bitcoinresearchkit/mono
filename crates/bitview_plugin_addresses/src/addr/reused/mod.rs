//! Reused address tracking.
//!
//! An address is "reused" if its lifetime `funded_txo_count > 1`, i.e.
//! it has received more than one output across its lifetime. This is
//! the simplest output-multiplicity proxy for address linkability.
//!
//! Two facets are tracked here:
//! - [`count`]: how many distinct addresses are currently reused
//!   (funded) and how many have *ever* been reused (total). Per address
//!   type plus an aggregated `all`.
//! - [`events`]: per-block address-reuse event counts on both sides.
//!   Output-side (`output_to_reused_addr_count`, outputs landing on
//!   addresses that had already received ≥ 1 prior output) and
//!   input-side (`input_from_reused_addr_count`, inputs spending from
//!   addresses with lifetime `funded_txo_count > 1`). Each count is
//!   paired with a percent over the matching block-level output/input
//!   total.

use bitview_cohort::ByAddrType;
use bitview_collections::Windows;
use bitview_plugin_inputs::ByTypeVecs as InputsByTypeVecs;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_plugin_outputs::ByTypeVecs;
use bitview_primitives::{Count, Lengths};
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Sats, Version};
use rayon::prelude::*;
use vecdb::{
    AnyStoredVec, Database, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, Rw, StorageMode,
};

use super::{
    count::AddrCountFundedTotalVecs,
    supply::{AddrSupplyShareVecs, AddrSupplyVecs},
};

mod events;

pub use events::{AddrEventsVecs, AddrTypeToAddrEventCount};

mod state;
mod type_state;

pub use state::ReusedAddrState;
pub use type_state::ReusedAddrTypeState;

/// Top-level container for all reused address tracking: counts (funded +
/// total), per-block reuse events (output-side + input-side), and funded
/// supply + share.
#[derive(Traversable)]
pub struct ReusedAddrVecs<M: StorageMode = Rw> {
    pub count: AddrCountFundedTotalVecs<M>,
    pub events: AddrEventsVecs<M>,
    /// Balance held at the represented block by funded addresses that satisfy
    /// an address predicate.
    pub supply: AddrSupplyVecs<M>,
    #[traversable(wrap = "supply", rename = "share")]
    /// Balance held by addresses satisfying an address predicate as a share
    /// of total supply; per-type variants divide by that address type's supply.
    pub supply_share: AddrSupplyShareVecs<M>,
}

impl ReusedAddrVecs {
    #[allow(clippy::too_many_arguments)]
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        outputs_by_type: &ByTypeVecs,
        inputs_by_type: &InputsByTypeVecs,
        active_addr_cumulative: &impl ReadableCloneableVec<Height, Count>,
        all_supply: &impl ReadableCloneableVec<Height, Sats>,
    ) -> Result<Self> {
        let count = AddrCountFundedTotalVecs::import(db, name, version, mappings)?;
        let events = AddrEventsVecs::import(
            db,
            name,
            version,
            mappings,
            window_starts,
            outputs_by_type,
            inputs_by_type,
            active_addr_cumulative,
        )?;
        let supply = AddrSupplyVecs::import(db, name, version, mappings, spot_price)?;
        let supply_share =
            AddrSupplyShareVecs::import(db, name, version, mappings, &supply, all_supply)?;

        Ok(Self {
            count,
            events,
            supply,
            supply_share,
        })
    }

    pub fn par_iter_stateful_height_mut(
        &mut self,
    ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.count
            .par_iter_height_mut()
            .chain(self.events.par_iter_height_mut())
            .chain(self.supply.par_iter_height_mut())
    }

    pub fn par_iter_height_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.count
            .par_iter_height_mut()
            .chain(self.events.par_iter_height_mut())
            .chain(self.supply.par_iter_height_mut())
            .chain(
                self.supply_share
                    .stored_vecs_mut()
                    .collect::<Vec<_>>()
                    .into_par_iter(),
            )
    }

    pub fn reset_height(&mut self) -> Result<()> {
        self.count.reset_height()?;
        self.events.reset_height()?;
        self.supply.reset_height()?;
        self.supply_share.reset_height()?;
        Ok(())
    }

    #[inline(always)]
    pub fn push_height(&mut self, state: &ReusedAddrState) {
        self.count.push_counts(&state.funded, &state.total);
        self.supply.push_supply(&state.supply);
        self.events.push_height(
            &state.output_events,
            &state.input_events,
            state.active.sum(),
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub fn compute_rest(
        &mut self,
        starting_lengths: &Lengths,
        type_supply_sats: &ByAddrType<&impl ReadableVec<Height, Sats>>,
        exit: &Exit,
    ) -> Result<()> {
        self.supply_share.compute_rest(
            starting_lengths.height,
            &self.supply,
            type_supply_sats,
            exit,
        )?;
        Ok(())
    }
}
