use brk_plugin::{Plugin, PluginGate};
use brk_traversable::Traversable;
use brk_types::{PartsPerMillion32, PartsPerMillion64, StoredF32};
use vecdb::{Database, Rw, StorageMode};

use super::dormancy_vecs::DormancyVecs;
use crate::internal::{LazyPerBlock, LazyRatioPerBlock, PerBlock, PercentPerBlock, RatioPerBlock};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    #[traversable(skip)]
    pub(crate) db: Database,
    pub puell_multiple: RatioPerBlock<PartsPerMillion64, M>,
    pub nvt: LazyRatioPerBlock<PartsPerMillion64>,
    pub gini: PercentPerBlock<PartsPerMillion32, M>,
    pub rhodl_ratio: RatioPerBlock<PartsPerMillion64, M>,
    pub thermo_cap_multiple: LazyRatioPerBlock<PartsPerMillion64>,
    pub coindays_destroyed_supply_adj: LazyPerBlock<StoredF32>,
    pub coinyears_destroyed_supply_adj: LazyPerBlock<StoredF32>,
    pub dormancy: DormancyVecs,
    pub stock_to_flow: LazyPerBlock<StoredF32>,
    pub seller_exhaustion: PerBlock<StoredF32, M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Send + Sync,
{
    fn id(&self) -> &'static str {
        super::DB_NAME
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
