use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::LazyWindowStartVec;
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, ReadableBoxedVec, Rw, StorageMode};

use super::SupplyChange;
use crate::families::Supply;

/// A cohort's supply, its change over each window and its share of all supply; `name` is its
/// supply series name (`utxos_1d_to_1w_old_supply`).
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
        name: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        all_supply: &ReadableBoxedVec<Height, Sats>,
    ) -> Result<Self> {
        let total = Supply::import(db, name, version, mappings, spot_price)?;
        let change = SupplyChange::new(
            name,
            version,
            &total.value,
            all_supply,
            mappings,
            window_starts,
        );
        Ok(Self { total, change })
    }
}
