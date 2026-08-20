use brk_error::Result;

use brk_types::{Cents, Height, Version};
use vecdb::{CachedBoxedVec, Database};

use super::{Vecs, sma::SmaVecs, vecs::EmaPeriodId};
use bitview_compute::{ColumnarPerBlock, LazyColumnPriceWithRatioPerBlock};

const EMA_VERSION: Version = Version::ONE;

pub fn forced_import(
    db: &Database,
    version: Version,
    mappings: &bitview_plugin_mappings::Vecs,
    blocks: &bitview_plugin_blocks::Vecs,
    spot_price: &CachedBoxedVec<Height, Cents>,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, mappings, blocks, spot_price)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
        blocks: &bitview_plugin_blocks::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let sma = SmaVecs::new(version, mappings, &blocks.lookback, spot_price.clone());
        let ema_version = version + EMA_VERSION;
        let ema = ColumnarPerBlock::forced_import(db, "price_ema_cents", ema_version, |source| {
            EmaPeriodId::series(|period| {
                LazyColumnPriceWithRatioPerBlock::new(
                    &format!("price_ema_{}", period.suffix()),
                    ema_version,
                    source,
                    period,
                    mappings,
                    spot_price,
                )
            })
        })?;

        Ok(Self { sma, ema })
    }
}
