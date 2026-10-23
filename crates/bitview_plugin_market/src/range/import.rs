use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::Price;
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::{Vecs, price_min_max_vecs::PriceMinMaxVecs};

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        Ok(Vecs {
            min: PriceMinMaxVecs {
                _1w: Price::import(db, "price_min_1w", version, mappings)?,
                _1m: Price::import(db, "price_min_1m", version, mappings)?,
                _1y: Price::import(db, "price_min_1y", version, mappings)?,
            },
            max: PriceMinMaxVecs {
                _1w: Price::import(db, "price_max_1w", version, mappings)?,
                _1m: Price::import(db, "price_max_1m", version, mappings)?,
                _1y: Price::import(db, "price_max_1y", version, mappings)?,
            },
            extrema: Default::default(),
        })
    }
}
