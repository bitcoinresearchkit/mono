use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::ValuePerBlockCumulative;
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::Vecs;

impl Vecs {
    pub fn import(db: &Database, version: Version, mappings: &MappingsVecs) -> Result<Self> {
        Ok(Self {
            total: ValuePerBlockCumulative::import(db, "unspendable_supply", version, mappings)?,
        })
    }
}
