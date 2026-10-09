use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{PerBlock, PercentPerBlock};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{
    Vecs,
    vecs::{HashPriceValueVecs, HashRateSmaVecs, RateVecs},
};

impl Vecs {
    pub(crate) fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let v5 = Version::new(5);
        let v8 = Version::new(8);

        Ok(Vecs {
            rate: RateVecs {
                block: PerBlock::import(db, "hash_rate", version + v5, mappings)?,
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
                block: PerBlock::import(db, "hash_price", version + v8, mappings)?,
                atl: PerBlock::import(db, "hash_price_atl", version + v8, mappings)?,
                rebound: PercentPerBlock::import(db, "hash_price_rebound", version + v8, mappings)?,
            },
            value: HashPriceValueVecs {
                block: PerBlock::import(db, "hash_value", version + v8, mappings)?,
                atl: PerBlock::import(db, "hash_value_atl", version + v8, mappings)?,
                rebound: PercentPerBlock::import(db, "hash_value_rebound", version + v8, mappings)?,
            },
        })
    }
}
