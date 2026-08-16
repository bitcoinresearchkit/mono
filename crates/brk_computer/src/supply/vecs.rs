use brk_plugin::{Plugin, PluginGate};
use brk_traversable::Traversable;
use brk_types::{Cents, CentsSigned, PartsPerMillionSigned64};
use vecdb::{Database, Rw, StorageMode};

use super::{burned, velocity};
use crate::internal::{
    LazyFiatPerBlock, LazyPerBlock, LazyPercentPerBlock, LazyRollingDeltasFiatFromHeight,
    LazySpotValuePerBlock, LazyValuePerBlock, Windows,
};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    pub(crate) plugin_gate: PluginGate,
    #[traversable(skip)]
    pub(crate) db: Database,

    pub circulating: LazyValuePerBlock,
    pub burned: burned::Vecs<M>,
    pub inflation_rate: LazyPercentPerBlock<PartsPerMillionSigned64>,
    pub velocity: velocity::Vecs,
    pub market_cap: LazyFiatPerBlock<Cents>,
    #[traversable(wrap = "market_cap", rename = "delta")]
    pub market_cap_delta:
        LazyRollingDeltasFiatFromHeight<Cents, CentsSigned, PartsPerMillionSigned64>,
    pub market_minus_realized_cap_growth_rate: Windows<LazyPerBlock<PartsPerMillionSigned64>>,
    pub hodled_or_lost: LazySpotValuePerBlock,
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
