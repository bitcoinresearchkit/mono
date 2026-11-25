use bitview_distribution::metrics::holding::Part;
use bitview_plugin_mappings::Vecs as Mappings;
use bitview_primitives::PartsPerMillion32;
use bitview_vecs::PercentPerBlock;
use brk_error::Result;
use brk_types::Version;
use vecdb::Database;

/// A part with its share stored (`{name}_share`), computed after the block loop.
pub(crate) fn import_part<T>(
    db: &Database,
    name: &str,
    v: Version,
    mappings: &Mappings,
    value: T,
) -> Result<Part<T, PercentPerBlock<PartsPerMillion32>>> {
    Ok(Part {
        value,
        share: PercentPerBlock::import(db, &format!("{name}_share"), v, mappings)?,
    })
}
