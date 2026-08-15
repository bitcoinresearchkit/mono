use brk_error::Result;
use brk_types::{Cents, Height, StoredF32, Version};
use vecdb::{Database, ReadableCloneableVec};

use super::{Vecs, vecs::PriceMinMaxVecs};
use crate::{
    indexes,
    internal::{
        CACHE_BUDGET, Identity, LazyLookbackVec, LazyPerBlock, PerBlock, PercentPerBlock, Price,
    },
};

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        indexes: &indexes::Vecs,
        spot_price: &(impl ReadableCloneableVec<Height, Cents> + 'static),
    ) -> Result<Self> {
        let v1 = Version::ONE;
        let v = version + v1;
        let true_range_source = LazyLookbackVec::new(
            "price_true_range_source",
            v,
            spot_price.read_only_boxed_clone(),
            1,
            |current, previous| {
                let previous = previous.unwrap_or(current);
                StoredF32::from((f64::from(current) - f64::from(previous)).abs())
            },
        );

        Ok(Self {
            min: PriceMinMaxVecs {
                _1w: Price::forced_import(db, "price_min_1w", version + v1, indexes)?,
                _2w: Price::forced_import(db, "price_min_2w", version + v1, indexes)?,
                _1m: Price::forced_import(db, "price_min_1m", version + v1, indexes)?,
                _1y: Price::forced_import(db, "price_min_1y", version + v1, indexes)?,
            },
            max: PriceMinMaxVecs {
                _1w: Price::forced_import(db, "price_max_1w", version + v1, indexes)?,
                _2w: Price::forced_import(db, "price_max_2w", version + v1, indexes)?,
                _1m: Price::forced_import(db, "price_max_1m", version + v1, indexes)?,
                _1y: Price::forced_import(db, "price_max_1y", version + v1, indexes)?,
            },
            true_range: LazyPerBlock::from_height_source::<Identity<StoredF32>>(
                "price_true_range",
                v,
                CACHE_BUDGET.wrap(true_range_source),
                indexes,
            ),
            true_range_sum_2w: PerBlock::forced_import(
                db,
                "price_true_range_sum_2w",
                version + v1,
                indexes,
            )?,
            choppiness_index_2w: PercentPerBlock::forced_import(
                db,
                "price_choppiness_index_2w",
                version + v1,
                indexes,
            )?,
        })
    }
}
