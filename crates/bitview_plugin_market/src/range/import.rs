use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyLookbackVec, LazyPerBlock, PerBlock, PercentPerBlock, Price};
use brk_error::Result;
use brk_types::{Cents, Height, StoredF32, Version};
use vecdb::{Database, Ident, ReadableCloneableVec};

use super::{Vecs, price_min_max_vecs::PriceMinMaxVecs};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Result<Self> {
        let v1 = Version::ONE;
        let v = version + v1;
        let true_range_source = LazyLookbackVec::new(
            "price_true_range_source",
            v,
            spot_price,
            1,
            |current, previous| {
                let previous = previous.unwrap_or(current);
                StoredF32::from((f64::from(current) - f64::from(previous)).abs())
            },
        );

        Ok(Vecs {
            min: PriceMinMaxVecs {
                _1w: Price::import(db, "price_min_1w", version + v1, mappings)?,
                _2w: Price::import(db, "price_min_2w", version + v1, mappings)?,
                _1m: Price::import(db, "price_min_1m", version + v1, mappings)?,
                _1y: Price::import(db, "price_min_1y", version + v1, mappings)?,
            },
            max: PriceMinMaxVecs {
                _1w: Price::import(db, "price_max_1w", version + v1, mappings)?,
                _2w: Price::import(db, "price_max_2w", version + v1, mappings)?,
                _1m: Price::import(db, "price_max_1m", version + v1, mappings)?,
                _1y: Price::import(db, "price_max_1y", version + v1, mappings)?,
            },
            true_range: LazyPerBlock::from_height_source::<Ident>(
                "price_true_range",
                v,
                &true_range_source,
                mappings,
            ),
            true_range_sum_2w: PerBlock::import(
                db,
                "price_true_range_sum_2w",
                version + v1,
                mappings,
            )?,
            choppiness_index_2w: PercentPerBlock::import(
                db,
                "price_choppiness_index_2w",
                version + v1,
                mappings,
            )?,
        })
    }
}
