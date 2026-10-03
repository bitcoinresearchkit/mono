use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::ValuePerBlockCumulative;
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

use super::Vecs;

impl Vecs {
    pub(crate) fn forced_import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
    ) -> Result<Self> {
        Ok(Vecs {
            op_return: ValuePerBlockCumulative::forced_import(
                db,
                "op_return_value",
                version,
                mappings,
            )?,
        })
    }
}
