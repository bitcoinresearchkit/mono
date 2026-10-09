use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_transforms::ThsToPhs;
use bitview_vecs::{LazyPerBlock, PerBlock, PercentPerBlock};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{
    Vecs,
    vecs::{HashPriceValueVecs, HashRateSmaVecs, RateVecs},
};

impl Vecs {
    pub(crate) fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let v4 = Version::new(4);
        let v5 = Version::new(5);
        let v6 = Version::new(6);
        let v7 = Version::new(7);

        let price_ths = PerBlock::import(db, "hash_price_ths", version + v4, mappings)?;
        let price_ths_min = PerBlock::import(db, "hash_price_ths_min", version + v6, mappings)?;
        let price_phs =
            LazyPerBlock::from_resolutions::<ThsToPhs>("hash_price_phs", version + v4, &price_ths);
        let price_phs_min = LazyPerBlock::from_resolutions::<ThsToPhs>(
            "hash_price_phs_min",
            version + v6,
            &price_ths_min,
        );

        let value_ths = PerBlock::import(db, "hash_value_ths", version + v4, mappings)?;
        let value_ths_min = PerBlock::import(db, "hash_value_ths_min", version + v6, mappings)?;
        let value_phs =
            LazyPerBlock::from_resolutions::<ThsToPhs>("hash_value_phs", version + v4, &value_ths);
        let value_phs_min = LazyPerBlock::from_resolutions::<ThsToPhs>(
            "hash_value_phs_min",
            version + v6,
            &value_ths_min,
        );

        Ok(Vecs {
            rate: RateVecs {
                base: PerBlock::import(db, "hash_rate", version + v5, mappings)?,
                sma: HashRateSmaVecs {
                    _1w: PerBlock::import(db, "hash_rate_sma_1w", version, mappings)?,
                    _1m: PerBlock::import(db, "hash_rate_sma_1m", version, mappings)?,
                    _2m: PerBlock::import(db, "hash_rate_sma_2m", version, mappings)?,
                    _1y: PerBlock::import(db, "hash_rate_sma_1y", version, mappings)?,
                },
                ath: PerBlock::import(db, "hash_rate_ath", version, mappings)?,
                drawdown: PercentPerBlock::import(db, "hash_rate_drawdown", version, mappings)?,
            },
            price: HashPriceValueVecs {
                ths: price_ths,
                ths_min: price_ths_min,
                phs: price_phs,
                phs_min: price_phs_min,
                rebound: PercentPerBlock::import(db, "hash_price_rebound", version + v7, mappings)?,
            },
            value: HashPriceValueVecs {
                ths: value_ths,
                ths_min: value_ths_min,
                phs: value_phs,
                phs_min: value_phs_min,
                rebound: PercentPerBlock::import(db, "hash_value_rebound", version + v7, mappings)?,
            },
        })
    }
}
