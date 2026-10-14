#![allow(clippy::type_complexity)]

mod compute;
mod dependencies;
mod endpoint;
mod has;
mod import;
mod ohlc;
mod oracle_feed;
mod split;
mod spot;

pub use dependencies::Dependencies;
pub use has::HasPrice;
pub use oracle_feed::{feed_blocks_for_warmup, feed_blocks_with};
pub use spot::SpotPrice;

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::Float64;
use bitview_traversable::Traversable;
use bitview_vecs::LazyPerBlock;
use brk_oracle::VERSION as ORACLE_VERSION;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, EagerVec, PcoVec, Rw, StorageMode};

use crate::{ohlc::OhlcPrice, split::SplitPrice};

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
    #[traversable(flatten)]
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
    /// a model-derived block price, not a contemporaneous exchange ticker. Zero
    /// before the first exchange price (block 74,641, July 2010), and so are USD
    /// values derived from it. At time-period indexes, the value is taken at the
    /// period's final block; null when the period has no block.
    pub spot: SpotPrice<M>,
    /// Whole satoshis one US dollar buys at the spot price, rounded down: 100,000,000 divided
    /// by the price in USD per BTC. 0 while the spot price is zero.
    sats_per_dollar: LazyPerBlock<Sats, Cents>,
    /// The oracle's exact reference bin at each height (the spot price before rounding to
    /// cents), so an update resumes the oracle where the previous one left it.
    #[traversable(hidden)]
    pub oracle_bin: M::Stored<EagerVec<PcoVec<Height, Float64>>>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}
