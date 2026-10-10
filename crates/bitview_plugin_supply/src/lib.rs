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
    LazyFiatPerBlock, LazyPerBlock, LazyPercentPerBlock, LazyRollingDeltasFiatFromHeight,
};
use brk_types::{Bitcoin, Cents, CentsSigned, Version};
use vecdb::{Database, Rw, StorageMode};

const STORAGE: PluginStorage = PluginStorage::new(PluginId::new("supply"), Version::new(10));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Total value of all unspent transaction outputs in the UTXO set; its USD
    /// value is the market cap.
    circulating: LazyPerBlock<Bitcoin, Bitcoin>,
    /// Value made provably unspendable: the genesis subsidy, `OP_RETURN` output
    /// values and unclaimed block rewards.
    burned: burned::Vecs<M>,
    /// Scheduled annual issuance over the circulating supply: the represented
    /// block's scheduled subsidy times 52,560 blocks, divided by the supply. The
    /// reciprocal of stock-to-flow. NaN while the supply is at most 50 BTC.
    pub inflation_rate: LazyPercentPerBlock<PartsPerMillionSigned64>,
    pub velocity: velocity::Vecs,
    /// Circulating supply valued at the represented block's Bitcoin spot price.
    #[traversable(wrap = "market_cap", rename = "block")]
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
    market_minus_realized_cap_growth_rate: Windows<LazyPercentPerBlock<PartsPerMillionSigned64>>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
