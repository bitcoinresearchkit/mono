use bitview_cohort::{AmountRange, SpendableType};
use bitview_distribution::metrics::ShareTotals;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use super::{CohortVecs, ZeroVecs};
use crate::state::UTXOStates;

/// UTXO cohorts, members first: amount ranges and spendable output types.
#[derive(Traversable)]
pub struct CohortMetrics<M: StorageMode = Rw> {
    /// UTXO amount bands classify each output by its own value.
    #[traversable(lead = "Unspent outputs, each holding")]
    pub amounts: Box<AmountRange<CohortVecs<M>, ZeroVecs<M>>>,
    pub types: Box<SpendableType<CohortVecs<M>>>,
}

impl CohortMetrics {
    pub fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        windows: &Windows<&LazyWindowStartVec>,
        spot: &ReadableBoxedVec<Height, Cents>,
        totals: ShareTotals<'_>,
    ) -> Result<Self> {
        let amounts = Box::new(AmountRange::try_from_fns(
            |_| ZeroVecs::import(db, version, mappings, windows),
            |id| CohortVecs::import(db, id.cohort(), version, mappings, windows, spot, totals),
        )?);
        let types = Box::new(SpendableType::try_from_fn(|id| {
            CohortVecs::import(db, id.cohort(), version, mappings, windows, spot, totals)
        })?);
        Ok(Self { amounts, types })
    }

    pub fn push(&mut self, states: &UTXOStates, price: Cents) {
        let (zero, bands) = self.amounts.split_mut();
        let (zero_state, band_states) = states.amount_range.split();
        zero.push(zero_state);
        for (vecs, state) in bands.into_iter().zip(band_states) {
            vecs.push(state, price);
        }
        for (vecs, state) in self.types.iter_mut().zip(states.type_.iter()) {
            vecs.push(state, price);
        }
    }

    /// Every stored vec of every cohort, in parallel across cohorts.
    pub fn par_iter_vecs_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        let (zero, bands) = self.amounts.split_mut();
        bands
            .into_par_iter()
            .chain(self.types.par_iter_mut())
            .flat_map_iter(|cohort| cohort.iter_any_stored_mut())
            .chain(zero.iter_any_stored_mut().collect::<Vec<_>>())
    }
}
