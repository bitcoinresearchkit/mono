#![allow(clippy::type_complexity)]

use bitview_plugin::{ImportContext, Plugin, PluginId, PluginStorage};
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{OhlcPrice, SplitPrice, SpotPrice};
use brk_error::Result;
use brk_oracle::VERSION as ORACLE_VERSION;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

mod compute;
mod oracle_feed;
pub use oracle_feed::{feed_blocks_for_warmup, feed_blocks_with};
mod dependencies;
mod has;

pub use dependencies::Dependencies;
pub use has::HasPrice;

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
    pub split: SplitPrice,
    /// Open-high-low-close (OHLC) candles formed from block-level Bitcoin spot
    /// prices within each supported time period. Heights before 340,000 use
    /// baked historical exchange prices; later heights use an on-chain oracle
    /// that estimates price from round-USD transaction-output patterns. Empty
    /// periods carry the previous close as all four candle values.
    pub ohlc: OhlcPrice,
    /// BRK's block-level Bitcoin (BTC/USD) spot-price estimate. Heights before
    /// 340,000 use baked historical exchange prices; later heights use an on-chain oracle
    /// that estimates price from round-USD transaction-output patterns. This is
    /// a model-derived block price, not a contemporaneous exchange ticker.
    pub spot: SpotPrice<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }
}

impl Vecs {
    pub fn import(context: ImportContext<'_>, mappings: &MappingsVecs) -> Result<Self> {
        let db = STORAGE.open_database(context, 100_000)?;
        let this = Self::forced_import_inner(&db, STORAGE.schema_version(), mappings)?;
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }

    fn forced_import_inner(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let spot = SpotPrice::forced_import(db, "price", version, mappings)?;
        let ohlc = OhlcPrice::from_spot("price_ohlc", version, mappings, &spot);
        let split = SplitPrice::new("price", version, mappings, &spot, &ohlc);

        Ok(Self {
            db: db.clone(),
            split,
            ohlc,
            spot,
        })
    }
}
