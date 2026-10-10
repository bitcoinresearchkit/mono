use bitview_cohort::{CohortContext, CohortId};
use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, ReadableBoxedVec, Rw, StorageMode};

use super::SupplyChange;
use crate::families::Supply;

/// A UTXO cohort's supply, its change over each window and its share of all supply.
#[derive(Traversable)]
pub struct CohortSupply<M: StorageMode = Rw> {
    /// Supply: amount of bitcoin held in the cohort's unspent transaction outputs.
    pub total: Supply<M>,
    #[traversable(flatten)]
    change: SupplyChange,
}

impl CohortSupply {
    pub fn import(
        db: &Database,
        cohort: CohortId,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let total = Supply::import(db, cohort, "supply", version, mappings, spot_price)?;
        let change = SupplyChange::new(
            CohortContext::Utxo,
            cohort,
            version,
            &total.value,
            all_supply,
            mappings,
            window_starts,
        );
        Ok(Self { total, change })
    }
}
