mod compute;
mod dependencies;
mod endpoint;
mod has;
mod import;
mod ohlc;
mod oracle_feed;
mod split;

pub use dependencies::Dependencies;
pub use has::HasPrice;
pub use oracle_feed::{block_payments, block_tally, feed_blocks_for_warmup, feed_blocks_with};

use bitview_plugin::{Plugin, PluginId, PluginStorage};
use bitview_primitives::Float64;
use bitview_traversable::Traversable;
use bitview_vecs::{LazyPerBlock, PerBlock, Price};
use brk_oracle::{Oracle, VERSION as ORACLE_VERSION};
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{Database, PcoVec, Rw, StorageMode};

use crate::{ohlc::OhlcPrice, split::SplitPrice};

const STORAGE: PluginStorage =
    PluginStorage::new(PluginId::new("price"), Version::new(20 + ORACLE_VERSION));
pub const ID: PluginId = STORAGE.id();

#[derive(Traversable, Plugin)]
#[plugin(storage = STORAGE)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    db: Database,

    /// Separate open, high, low, and close views of BRK's block-level BTC/USD
    /// price series for each supported time period. Heights before 340,000 use
    /// baked historical exchange prices; later heights use an on-chain oracle
    /// that estimates price from round-USD transaction-output patterns. Zero
    /// before the first exchange price (block 74,641, August 2010).
    #[traversable(flatten)]
    split: SplitPrice,
    /// Open-high-low-close (OHLC) candles formed from block-level Bitcoin spot
    /// prices within each supported time period. Heights before 340,000 use
    /// baked historical exchange prices; later heights use an on-chain oracle
    /// that estimates price from round-USD transaction-output patterns. Empty
    /// periods carry the previous close as all four candle values. Blocks before
    /// the first price (block 74,641) are left out: the period holding it starts
    /// there, and earlier periods are flat zero candles.
    #[traversable(lead = "Candles: each period's opening, highest, lowest and closing prices.")]
    ohlc: OhlcPrice,
    /// BRK's block-level Bitcoin (BTC/USD) spot-price estimate. Heights before
    /// 340,000 use baked historical exchange prices; later heights use an on-chain oracle
    /// that estimates price from round-USD transaction-output patterns. This is
    /// a model-derived block price, not a contemporaneous exchange ticker. Before
    /// the first exchange price (block 74,641, August 2010), this price and USD
    /// amounts valued with it are zero. At time-period indexes, the value is taken
    /// at the period's final block; null when the period has no block.
    #[traversable(lead = "Bitcoin's price in US dollars, block by block: from exchanges' history at first, then estimated from the chain itself.")]
    pub spot: Price<PerBlock<Cents, M>>,
    /// Whole satoshis one US dollar buys at the spot price, rounded down: 100,000,000 divided
    /// by the price in USD per BTC. 0 while the spot price is zero.
    #[traversable(lead = "Sats per dollar: how many satoshis one US dollar buys.")]
    sats_per_dollar: LazyPerBlock<Sats, Cents>,
    /// The oracle's exact reference bin at each height (the spot price before rounding to
    /// cents), so an update resumes the oracle where the previous one left it.
    #[traversable(hidden)]
    pub oracle_bin: M::Stored<PcoVec<Height, Float64>>,
    /// The oracle after the last computed height (the length it continues at): the next update
    /// resumes it instead of rebuilding it from the stored bin and its warmup blocks.
    oracle: M::WriteOnly<Option<(usize, Oracle)>>,
}
