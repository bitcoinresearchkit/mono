#![allow(clippy::type_complexity)]

mod compute;
mod dependencies;
mod has;
mod import;
mod oracle_feed;

pub use dependencies::Dependencies;
pub use has::HasPrice;
pub use oracle_feed::{feed_blocks_for_warmup, feed_blocks_with};

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, OhlcPrice, SplitPrice, SpotPrice};
use brk_oracle::VERSION as ORACLE_VERSION;
use brk_types::{Cents, Sats, Version};
use vecdb::{Database, Rw, StorageMode};

const STORAGE: PluginStorage =
    PluginStorage::new(PluginId::new("price"), Version::new(20 + ORACLE_VERSION));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Separate open, high, low, and close views of BRK's block-level BTC/USD
    /// price series for each supported time period. Heights before 340,000 use
    /// baked historical exchange prices; later heights use an on-chain oracle
    /// that estimates price from round-USD transaction-output patterns.
    split: SplitPrice,
    /// Open-high-low-close (OHLC) candles formed from block-level Bitcoin spot
    /// prices within each supported time period. Heights before 340,000 use
    /// baked historical exchange prices; later heights use an on-chain oracle
    /// that estimates price from round-USD transaction-output patterns. Empty
    /// periods carry the previous close as all four candle values.
    ohlc: OhlcPrice,
    /// BRK's block-level Bitcoin (BTC/USD) spot-price estimate. Heights before
    /// 340,000 use baked historical exchange prices; later heights use an on-chain oracle
    /// that estimates price from round-USD transaction-output patterns. This is
    /// a model-derived block price, not a contemporaneous exchange ticker.
    pub spot: SpotPrice<M>,
    /// Whole satoshis one US dollar buys at the spot price: 100,000,000 divided
    /// by the price in USD per BTC.
    sats_per_dollar: LazyPerBlock<Sats, Cents>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
