use bitview_plugin::{Plugin, PluginGate, PluginId};
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

    /// Total value of all unspent transaction outputs in the UTXO set.
    pub circulating: LazyValuePerBlock,
    /// Cumulative provably unspendable supply from the genesis subsidy,
    /// `OP_RETURN` output values, and unclaimed block rewards.
    pub burned: burned::Vecs<M>,
    /// Change in circulating supply from the first block in the trailing
    /// 365-day monotonic-time window through the current block, divided by the
    /// starting supply. Returns NaN while the starting supply is at most 50 BTC.
    pub inflation_rate: LazyPercentPerBlock<PartsPerMillionSigned64>,
    pub velocity: velocity::Vecs,
    /// Circulating supply valued at the current Bitcoin spot price.
    pub market_cap: LazyFiatPerBlock<Cents>,
    /// Absolute and relative change in market capitalization from the first
    /// block in each named trailing monotonic-time window.
    #[traversable(wrap = "market_cap", rename = "delta")]
    pub market_cap_delta:
        LazyRollingDeltasFiatFromHeight<Cents, CentsSigned, PartsPerMillionSigned64>,
    /// Market-cap growth rate minus realized-cap growth rate over the named
    /// trailing monotonic-time window. A component with a zero starting value
    /// contributes zero growth.
    pub market_minus_realized_cap_growth_rate: Windows<LazyPerBlock<PartsPerMillionSigned64>>,
    /// Circulating supply multiplied by cointime vaultedness, which is one
    /// minus liveliness, and valued at the current Bitcoin spot price.
    pub hodled_or_lost: LazySpotValuePerBlock,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Send + Sync,
{
    fn id(&self) -> PluginId {
        PluginId::new(super::DB_NAME)
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
