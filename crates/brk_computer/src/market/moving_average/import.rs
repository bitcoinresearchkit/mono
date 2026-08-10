use brk_error::Result;
use brk_types::{Cents, Height, Version};
use vecdb::{CachedBoxedVec, Database};

use super::{Vecs, sma::SmaVecs, vecs::EmaPeriodId};
use crate::{
    blocks, indexes,
    internal::{ColumnarPerBlock, LazyColumnPriceWithRatioPerBlock},
};

const EMA_VERSION: Version = Version::ONE;

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        blocks: &blocks::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let sma = SmaVecs::new(version, indexes, &blocks.lookback, spot_price.clone());
        let ema_version = version + EMA_VERSION;
        let ema = ColumnarPerBlock::forced_import(db, "price_ema_cents", ema_version, |source| {
            EmaPeriodId::series(|period| {
                LazyColumnPriceWithRatioPerBlock::new(
                    &format!("price_ema_{}", period.suffix()),
                    ema_version,
                    source,
                    period,
                    indexes,
                    spot_price,
                )
            })
        })?;

        Ok(Self { sma, ema })
    }
}
