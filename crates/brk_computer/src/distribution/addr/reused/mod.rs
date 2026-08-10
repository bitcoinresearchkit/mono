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

mod events;

pub use events::{AddrEventsVecs, AddrTypeToAddrEventCount};

use brk_cohort::ByAddrType;
use brk_error::Result;
use brk_indexer::Lengths;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, Sats, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, CachedBoxedVec, Database, Exit, ReadableVec, Rw, StorageMode};

use super::{
    count::AddrCountFundedTotalVecs,
    supply::{AddrSupplyShareVecs, AddrSupplyVecs},
};
use crate::{
    distribution::metrics::AllSupplyCache,
    indexes, inputs,
    internal::{CachedWindowStartVec, Windows},
    outputs,
};

mod state;

pub use state::ReusedAddrState;

/// Top-level container for all reused address tracking: counts (funded +
/// total), per-block reuse events (output-side + input-side), and funded
/// supply + share.
#[derive(Traversable)]
pub struct ReusedAddrVecs<M: StorageMode = Rw> {
    pub count: AddrCountFundedTotalVecs<M>,
    pub events: AddrEventsVecs<M>,
    pub supply: AddrSupplyVecs<M>,
    #[traversable(wrap = "supply", rename = "share")]
    pub supply_share: AddrSupplyShareVecs<M>,
}

impl ReusedAddrVecs {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        spot_price: &CachedBoxedVec<Height, Cents>,
        outputs_by_type: &outputs::ByTypeVecs,
        inputs_by_type: &inputs::ByTypeVecs,
        all_supply: &AllSupplyCache,
    ) -> Result<Self> {
        let count = AddrCountFundedTotalVecs::forced_import(db, name, version, indexes)?;
        let events = AddrEventsVecs::forced_import(
            db,
            name,
            version,
            indexes,
            cached_starts,
            outputs_by_type,
            inputs_by_type,
        )?;
        let supply = AddrSupplyVecs::forced_import(db, name, version, indexes, spot_price)?;
        let supply_share =
            AddrSupplyShareVecs::forced_import(db, name, version, indexes, &supply, all_supply)?;

        Ok(Self {
            count,
            events,
            supply,
            supply_share,
        })
    }

    pub(crate) fn min_stateful_len(&self) -> usize {
        self.count
            .min_stateful_len()
            .min(self.events.min_stateful_len())
            .min(self.supply.min_stateful_len())
    }

    pub(crate) fn par_iter_stateful_height_mut(
        &mut self,
    ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.count
            .par_iter_height_mut()
            .chain(self.events.par_iter_height_mut())
            .chain(self.supply.par_iter_height_mut())
    }

    pub(crate) fn par_iter_height_mut(
        &mut self,
    ) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        self.count
            .par_iter_height_mut()
            .chain(self.events.par_iter_height_mut())
            .chain(self.supply.par_iter_height_mut())
            .chain(rayon::iter::once(self.supply_share.stored_mut()))
    }

    pub(crate) fn reset_height(&mut self) -> Result<()> {
        self.count.reset_height()?;
        self.events.reset_height()?;
        self.supply.reset_height()?;
        self.supply_share.reset_height()?;
        Ok(())
    }

    #[inline(always)]
    pub(crate) fn push_height(&mut self, state: &ReusedAddrState, active_addr_count: u32) {
        let active_reused_addr_count = state.active.sum();
        debug_assert!(u32::try_from(active_reused_addr_count).is_ok());

        self.count.push_counts(&state.funded, &state.total);
        self.supply.push_supply(&state.supply);
        self.events.push_height(
            &state.output_events,
            &state.input_events,
            active_addr_count,
            active_reused_addr_count as u32,
        );
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn compute_rest(
        &mut self,
        starting_lengths: &Lengths,
        type_supply_sats: &ByAddrType<&impl ReadableVec<Height, Sats>>,
        exit: &Exit,
    ) -> Result<()> {
        self.events.compute_rest(starting_lengths, exit)?;
        self.supply_share.compute_rest(
            starting_lengths.height,
            &self.supply,
            type_supply_sats,
            exit,
        )?;
        Ok(())
    }
}
