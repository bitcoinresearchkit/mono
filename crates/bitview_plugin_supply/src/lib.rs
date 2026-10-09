mod burned;
mod compute;
mod dependencies;
mod import;
mod velocity;

pub use dependencies::Dependencies;

use bitview_collections::Windows;
use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::PartsPerMillionSigned64;
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyFiatPerBlock, LazyFixedRatioPerBlock, LazyPerBlock, LazyRollingDeltasFiatFromHeight,
    LazySpotValuePerBlock, LazyValuePerBlock,
};
use brk_types::{Cents, CentsSigned, Version};
use vecdb::{Database, Rw, StorageMode};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("supply"), Version::new(10));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Total value of all unspent transaction outputs in the UTXO set.
    circulating: LazyValuePerBlock,
    /// Cumulative provably unspendable supply from the genesis subsidy,
    /// `OP_RETURN` output values, and unclaimed block rewards.
    burned: burned::Vecs<M>,
    /// Change in circulating supply from the first block in the trailing
    /// 365-day monotonic-time window through the represented block, divided by
    /// the starting supply. Returns NaN while the starting supply is at most 50
    /// BTC.
    pub inflation_rate: LazyFixedRatioPerBlock<PartsPerMillionSigned64>,
    pub velocity: velocity::Vecs,
    /// Circulating supply valued at the represented block's Bitcoin spot price.
    #[traversable(wrap = "market_cap", rename = "usd")]
    market_cap: LazyFiatPerBlock<Cents>,
    /// Absolute and relative change in market capitalization from the first
    /// block in each supported trailing monotonic-time window.
    #[traversable(wrap = "market_cap", rename = "delta")]
    market_cap_delta: LazyRollingDeltasFiatFromHeight<Cents, CentsSigned, PartsPerMillionSigned64>,
    /// Market-cap growth rate minus realized-cap growth rate over a
    /// trailing monotonic-time window. Realized cap values each unspent output
    /// at Bitcoin's spot price when it was created. Positive values mean market
    /// value grew faster than this creation-date capital base; negative values
    /// mean it grew more slowly. A component with a zero starting value
    /// contributes zero growth.
    market_minus_realized_cap_growth_rate: Windows<LazyPerBlock<PartsPerMillionSigned64>>,
    /// Circulating supply multiplied by cointime vaultedness, which is one
    /// minus liveliness, and valued at the represented block's Bitcoin spot
    /// price. Liveliness is cumulative coinblocks destroyed divided by
    /// cumulative coinblocks created, so this estimates the market value of
    /// supply associated with holding time that remains stored rather than
    /// consumed by spending.
    hodled_or_lost: LazySpotValuePerBlock,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
