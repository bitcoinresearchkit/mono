use brk_error::Result;

use brk_types::Version;
use vecdb::Database;

use super::Vecs;
use bitview_compute::{PerBlock, PercentPerBlock};

pub fn forced_import(
    db: &Database,
    version: Version,
    indexes: &bitview_plugin_indexes::Vecs,
) -> Result<Vecs> {
    Vecs::forced_import(db, version, indexes)
}

impl Vecs {
    fn forced_import(
        db: &Database,
        version: Version,
        indexes: &bitview_plugin_indexes::Vecs,
    ) -> Result<Self> {
        Ok(Self {
            inflation_rate: PercentPerBlock::forced_import(
                db,
                "cointime_adj_inflation_rate",
                version + Version::TWO,
                indexes,
            )?,
            tx_velocity_native: PerBlock::forced_import(
                db,
                "cointime_adj_tx_velocity_btc",
                version,
                indexes,
            )?,
            tx_velocity_fiat: PerBlock::forced_import(
                db,
                "cointime_adj_tx_velocity_usd",
                version,
                indexes,
            )?,
        })
    }
}
