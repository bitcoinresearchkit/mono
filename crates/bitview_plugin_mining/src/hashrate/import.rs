use brk_error::Result;

use bitview_compute::{LazyPerBlock, PerBlock, PercentPerBlock, ThsToPhsF32};
use brk_types::Version;
use vecdb::{Database, ReadableCloneableVec};

use super::{
    Vecs,
    vecs::{HashPriceValueVecs, HashRateSmaVecs, RateVecs},
};

pub fn forced_import(
    db: &Database,
    version: Version,
    mappings: &bitview_plugin_mappings::Vecs,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, mappings)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> Result<Self> {
        let v4 = Version::new(4);
        let v5 = Version::new(5);
        let v6 = Version::new(6);

        let price_ths = PerBlock::forced_import(db, "hash_price_ths", version + v4, mappings)?;
        let price_ths_min =
            PerBlock::forced_import(db, "hash_price_ths_min", version + v6, mappings)?;
        let price_phs = LazyPerBlock::from_computed::<ThsToPhsF32>(
            "hash_price_phs",
            version + v4,
            price_ths.height.read_only_boxed_clone(),
            &price_ths,
        );
        let price_phs_min = LazyPerBlock::from_computed::<ThsToPhsF32>(
            "hash_price_phs_min",
            version + v6,
            price_ths_min.height.read_only_boxed_clone(),
            &price_ths_min,
        );

        let value_ths = PerBlock::forced_import(db, "hash_value_ths", version + v4, mappings)?;
        let value_ths_min =
            PerBlock::forced_import(db, "hash_value_ths_min", version + v6, mappings)?;
        let value_phs = LazyPerBlock::from_computed::<ThsToPhsF32>(
            "hash_value_phs",
            version + v4,
            value_ths.height.read_only_boxed_clone(),
            &value_ths,
        );
        let value_phs_min = LazyPerBlock::from_computed::<ThsToPhsF32>(
            "hash_value_phs_min",
            version + v6,
            value_ths_min.height.read_only_boxed_clone(),
            &value_ths_min,
        );

        Ok(Self {
            rate: RateVecs {
                base: PerBlock::forced_import(db, "hash_rate", version + v5, mappings)?,
                sma: HashRateSmaVecs {
                    _1w: PerBlock::forced_import(db, "hash_rate_sma_1w", version, mappings)?,
                    _1m: PerBlock::forced_import(db, "hash_rate_sma_1m", version, mappings)?,
                    _2m: PerBlock::forced_import(db, "hash_rate_sma_2m", version, mappings)?,
                    _1y: PerBlock::forced_import(db, "hash_rate_sma_1y", version, mappings)?,
                },
                ath: PerBlock::forced_import(db, "hash_rate_ath", version, mappings)?,
                drawdown: PercentPerBlock::forced_import(
                    db,
                    "hash_rate_drawdown",
                    version,
                    mappings,
                )?,
            },
            price: HashPriceValueVecs {
                ths: price_ths,
                ths_min: price_ths_min,
                phs: price_phs,
                phs_min: price_phs_min,
                rebound: PercentPerBlock::forced_import(
                    db,
                    "hash_price_rebound",
                    version + v6,
                    mappings,
                )?,
            },
            value: HashPriceValueVecs {
                ths: value_ths,
                ths_min: value_ths_min,
                phs: value_phs,
                phs_min: value_phs_min,
                rebound: PercentPerBlock::forced_import(
                    db,
                    "hash_value_rebound",
                    version + v6,
                    mappings,
                )?,
            },
        })
    }
}
