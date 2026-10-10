use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{LazySpotValuePerBlock, LazyWindowStartVec};
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec};

use super::SupplyChange;

/// A cohort's supply views over its stored sats, the mirror of `CapitalViews`: the supply, its
/// change over each window and its share. `name` is the supply series name (`veteran_supply`).
#[derive(Clone, Traversable)]
pub struct SupplyViews {
    /// Supply: amount of bitcoin held in the cohort's unspent transaction outputs.
    total: LazySpotValuePerBlock,
    #[traversable(flatten)]
    change: SupplyChange,
}

impl SupplyViews {
    pub fn new(
        name: &str,
        version: Version,
        supply: &impl ReadableCloneableVec<Height, Sats>,
        all_supply: &impl ReadableCloneableVec<Height, Sats>,
        spot_price: &ReadableBoxedVec<Height, Cents>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        let total =
            LazySpotValuePerBlock::from_sats_source(name, version, supply, mappings, spot_price);
        Self {
            change: SupplyChange::new(name, version, &total, all_supply, mappings, window_starts),
            total,
        }
    }
}
