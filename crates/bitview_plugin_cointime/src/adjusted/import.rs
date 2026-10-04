use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{FixedRatioPerBlock, PerBlock};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::Vecs;

impl Vecs {
    pub(crate) fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        Ok(Vecs {
            inflation_rate: FixedRatioPerBlock::import(
                db,
                "cointime_adj_inflation_rate",
                version + Version::new(3),
                mappings,
            )?,
            tx_velocity_native: PerBlock::import(
                db,
                "cointime_adj_tx_velocity_btc",
                version,
                mappings,
            )?,
            tx_velocity_fiat: PerBlock::import(
                db,
                "cointime_adj_tx_velocity_usd",
                version,
                mappings,
            )?,
        })
    }
}
