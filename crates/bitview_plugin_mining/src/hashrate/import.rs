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
        let v6 = Version::new(6);
        let v8 = Version::new(8);
        let v9 = Version::new(9);

        Ok(Vecs {
            rate: RateVecs {
                block: PerBlock::import(db, "hashrate", version + v6, mappings)?,
                sma: HashRateSmaVecs {
                    _1w: PerBlock::import(db, "hashrate_sma_1w", version, mappings)?,
                    _1m: PerBlock::import(db, "hashrate_sma_1m", version, mappings)?,
                    _2m: PerBlock::import(db, "hashrate_sma_2m", version, mappings)?,
                    _1y: PerBlock::import(db, "hashrate_sma_1y", version, mappings)?,
                    windows: Default::default(),
                },
                ath: PerBlock::import(db, "hashrate_ath", version, mappings)?,
                drawdown: PercentPerBlock::import(db, "hashrate_drawdown", version, mappings)?,
            },
            price: HashPriceValueVecs {
                block: PerBlock::import(db, "hash_price", version + v9, mappings)?,
                atl: PerBlock::import(db, "hash_price_atl", version + v9, mappings)?,
                rebound: PercentPerBlock::import(db, "hash_price_rebound", version + v9, mappings)?,
            },
            value: HashPriceValueVecs {
                block: PerBlock::import(db, "hash_value", version + v8, mappings)?,
                atl: PerBlock::import(db, "hash_value_atl", version + v8, mappings)?,
                rebound: PercentPerBlock::import(db, "hash_value_rebound", version + v8, mappings)?,
            },
        })
    }
}
