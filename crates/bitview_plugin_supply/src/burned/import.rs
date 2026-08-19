use brk_error::Result;

use brk_types::Version;
use vecdb::Database;

use super::Vecs;
use bitview_compute::ValuePerBlockCumulative;

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
            total: ValuePerBlockCumulative::forced_import(
                db,
                "unspendable_supply",
                version,
                indexes,
            )?,
        })
    }
}
