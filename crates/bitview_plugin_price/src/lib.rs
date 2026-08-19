#![allow(clippy::type_complexity)]

mod by_unit;
mod compute;
mod dependencies;
mod has;
mod lazy_ohlc;
mod ohlcs;

use brk_error::Result;

use std::path::Path;

use bitview_compute::{
    CACHE_BUDGET, CachedPerBlock, CentsUnsignedToDollars, CentsUnsignedToSats, LazyIndexes,
    LazyPerBlock, OhlcCentsToDollars, OhlcCentsToHighCents, OhlcCentsToLowCents,
    OhlcCentsToOpenCents, OhlcCentsToSats, Resolutions,
    db_utils::{finalize_db, open_db},
};
use bitview_plugin::{Plugin, PluginGate, PluginId};
use bitview_traversable::Traversable;
use brk_oracle::VERSION as ORACLE_VERSION;
use brk_types::Version;
use vecdb::{Database, Rw, StorageMode};

use by_unit::{OhlcByUnit, PriceByUnit, SplitByUnit, SplitCloseByUnit, SplitIndexesByUnit};
pub use dependencies::Dependencies;
pub use has::HasPrice;
use ohlcs::{LazyIndexesFromOhlc, LazyOhlcCentsVecs, LazyOhlcVecs};

pub const ID: PluginId = PluginId::new("price");

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,

    pub split: SplitByUnit,
    /// OHLC candles formed from block-level Bitcoin spot prices within each
    /// supported time period. Empty periods carry the previous close as all
    /// four candle values.
    pub ohlc: OhlcByUnit,
    /// Bitcoin spot price assigned to each block.
    pub spot: PriceByUnit<M>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn id(&self) -> PluginId {
        ID
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}

impl Vecs {
    pub fn forced_import(
        parent: &Path,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
    ) -> Result<Self> {
        let db = open_db(parent, ID.as_str(), 100_000)?;
        let this = Self::forced_import_inner(&db, version, indexes)?;
        finalize_db(&this.db, &this)?;
        Ok(this)
    }

    fn forced_import_inner(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
    ) -> Result<Self> {
        // `ORACLE_VERSION` folds in the on-chain oracle algorithm version so
        // every price-derived module invalidates when computed prices change.
        let version = version + Version::new(11 + ORACLE_VERSION);

        let price_cents = CachedPerBlock::forced_import(db, "price_cents", version, indexes)?;
        let close_cents = Resolutions::from_boxed_height_source(
            "price_close_cents",
            price_cents.height.read_only_boxed_clone(),
            version,
            indexes,
        );

        let ohlc_cents = LazyOhlcCentsVecs::new(
            "price_ohlc_cents",
            version,
            indexes,
            price_cents.height.read_only_cached_boxed_clone(),
        );

        let open_cents = LazyIndexes::from_ohlc_indexes::<OhlcCentsToOpenCents>(
            "price_open_cents",
            version,
            &ohlc_cents,
        );
        let high_cents = LazyIndexes::from_ohlc_indexes::<OhlcCentsToHighCents>(
            "price_high_cents",
            version,
            &ohlc_cents,
        );
        let low_cents = LazyIndexes::from_ohlc_indexes::<OhlcCentsToLowCents>(
            "price_low_cents",
            version,
            &ohlc_cents,
        );

        let price_usd = LazyPerBlock::from_cached_computed::<CentsUnsignedToDollars>(
            "price",
            version,
            price_cents.height.read_only_boxed_clone(),
            &price_cents,
        );

        let open_usd = LazyIndexes::from_lazy_indexes::<CentsUnsignedToDollars, _>(
            "price_open",
            version,
            &open_cents,
        );
        let high_usd = LazyIndexes::from_lazy_indexes::<CentsUnsignedToDollars, _>(
            "price_high",
            version,
            &high_cents,
        );
        let low_usd = LazyIndexes::from_lazy_indexes::<CentsUnsignedToDollars, _>(
            "price_low",
            version,
            &low_cents,
        );

        let source = CACHE_BUDGET.wrap(price_usd.height.clone());
        let close_usd = Resolutions::from_height_source("price_close", source, version, indexes);

        let ohlc_usd = LazyOhlcVecs::from_ohlc_indexes::<OhlcCentsToDollars>(
            "price_ohlc",
            version,
            &ohlc_cents,
        );

        let price_sats = LazyPerBlock::from_cached_computed::<CentsUnsignedToSats>(
            "price_sats",
            version,
            price_cents.height.read_only_boxed_clone(),
            &price_cents,
        );

        let open_sats = LazyIndexes::from_lazy_indexes::<CentsUnsignedToSats, _>(
            "price_open_sats",
            version,
            &open_cents,
        );
        // Sats are inversely related to cents (sats = 10B/cents), so high↔low are swapped
        let high_sats = LazyIndexes::from_lazy_indexes::<CentsUnsignedToSats, _>(
            "price_high_sats",
            version,
            &low_cents,
        );
        let low_sats = LazyIndexes::from_lazy_indexes::<CentsUnsignedToSats, _>(
            "price_low_sats",
            version,
            &high_cents,
        );

        let source = CACHE_BUDGET.wrap(price_sats.height.clone());
        let close_sats =
            Resolutions::from_height_source("price_close_sats", source, version, indexes);

        // OhlcCentsToSats handles the high↔low swap internally
        let ohlc_sats = LazyOhlcVecs::from_ohlc_indexes::<OhlcCentsToSats>(
            "price_ohlc_sats",
            version,
            &ohlc_cents,
        );

        let split = SplitByUnit {
            open: SplitIndexesByUnit {
                usd: open_usd,
                cents: open_cents,
                sats: open_sats,
            },
            high: SplitIndexesByUnit {
                usd: high_usd,
                cents: high_cents,
                sats: high_sats,
            },
            low: SplitIndexesByUnit {
                usd: low_usd,
                cents: low_cents,
                sats: low_sats,
            },
            close: SplitCloseByUnit {
                usd: close_usd,
                cents: close_cents,
                sats: close_sats,
            },
        };

        let ohlc = OhlcByUnit {
            usd: ohlc_usd,
            cents: ohlc_cents,
            sats: ohlc_sats,
        };

        let spot = PriceByUnit {
            usd: price_usd,
            cents: price_cents,
            sats: price_sats,
        };

        Ok(Self {
            plugin_gate: Default::default(),
            db: db.clone(),
            split,
            ohlc,
            spot,
        })
    }
}
