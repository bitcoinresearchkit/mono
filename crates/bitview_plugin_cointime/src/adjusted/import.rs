use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{PerBlock, PercentPerBlock};
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{Vecs, Velocity};

impl Vecs {
    pub(crate) fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        let inflation_rate = PercentPerBlock::import(
            db,
            "cointime_adjusted_inflation_rate",
            version + Version::new(3),
            mappings,
        )?;
        let version = version + Version::new(4);
        Ok(Vecs {
            inflation_rate,
            velocity: Velocity {
                btc: PerBlock::import(db, "cointime_adjusted_velocity_btc", version, mappings)?,
                usd: PerBlock::import(db, "cointime_adjusted_velocity_usd", version, mappings)?,
            },
        })
    }
}
