use bitview_cohort::{AmountRange, SpendableType};
use bitview_collections::Windows;
use bitview_distribution::metrics::ShareTotals;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazySpotValuePerBlock, LazyWindowStartVec, import_cached};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use rayon::prelude::*;
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode, WritableVec};

use super::{CohortVecs, TypeVecs};
use crate::state::UTXOStates;

/// UTXO cohorts, members first: amount ranges and spendable output types.
#[derive(Traversable)]
pub struct CohortMetrics<M: StorageMode = Rw> {
    pub amounts: Box<AmountRange<CohortVecs<M>>>,
    pub types: Box<SpendableType<TypeVecs<M>>>,
    /// Mean unspent output value across every spendable type, calculated from
    /// the same block's supply and count.
    pub avg_amount: LazySpotValuePerBlock,
    #[traversable(hidden)]
    avg_amount_sats: CachedSeries<Height, Sats, M>,
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
            TypeVecs::import(db, cohort, version, mappings, windows, spot, totals)
        })?);
        let avg_amount_sats = import_cached(db, "avg_utxo_amount_sats", version)?;
        let avg_amount = LazySpotValuePerBlock::from_sats_source(
            "avg_utxo_amount",
            version,
            &avg_amount_sats,
            mappings,
            spot,
        );
        Ok(Self {
            amounts,
            types,
            avg_amount,
            avg_amount_sats,
        })
    }

    pub fn push(&mut self, states: &UTXOStates, price: Cents) {
        for (vecs, state) in self.amounts.iter_mut().zip(states.amount_range.iter()) {
            vecs.push(state, price);
        }
        let mut supply = Sats::ZERO;
        let mut count = 0u64;
        for (vecs, state) in self.types.iter_mut().zip(states.type_.iter()) {
            vecs.push(state, price);
            supply += state.supply_value();
            count += u64::from(state.output_counts().0);
        }
        self.avg_amount_sats.push(supply / Count::from(count));
    }

    pub fn par_iter_vecs_mut(&mut self) -> impl ParallelIterator<Item = &mut dyn AnyStoredVec> {
        let mut vecs: Vec<&mut dyn AnyStoredVec> = Vec::with_capacity(512);
        for cohort in self.amounts.iter_mut() {
            vecs.extend(cohort.stored_vecs_mut());
        }
        for cohort in self.types.iter_mut() {
            vecs.extend(cohort.stored_vecs_mut());
        }
        vecs.push(&mut self.avg_amount_sats);
        vecs.into_par_iter()
    }
}
