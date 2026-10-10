use bitview_cohort::{AmountRange, SpendableType};
use bitview_collections::Windows;
use bitview_distribution::metrics::ShareTotals;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Height, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode};

use super::CohortVecs;
use crate::state::UTXOStates;

/// UTXO cohorts, members first: amount ranges and spendable output types.
#[derive(Traversable)]
pub struct CohortMetrics<M: StorageMode = Rw> {
    pub amounts: Box<AmountRange<CohortVecs<M>>>,
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
        let amounts = Box::new(AmountRange::try_new(|cohort| {
            CohortVecs::import(db, cohort, version, mappings, windows, spot, totals)
        })?);
        let types = Box::new(SpendableType::try_new(|cohort| {
            CohortVecs::import(db, cohort, version, mappings, windows, spot, totals)
        })?);
        Ok(Self { amounts, types })
    }

    pub fn push(&mut self, states: &UTXOStates, price: Cents) {
        for (vecs, state) in self.amounts.iter_mut().zip(states.amount_range.iter()) {
            vecs.push(state, price);
        }
        for (vecs, state) in self.types.iter_mut().zip(states.type_.iter()) {
            vecs.push(state, price);
        }
    }

    pub fn par_iter_vecs_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = Vec::with_capacity(512);
        for cohort in self.amounts.iter_mut() {
            vecs.extend(cohort.stored_vecs_mut());
        }
        for cohort in self.types.iter_mut() {
            vecs.extend(cohort.stored_vecs_mut());
        }
        vecs.into_par_iter()
    }
}
