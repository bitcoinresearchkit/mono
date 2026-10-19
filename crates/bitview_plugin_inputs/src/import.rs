use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{
    LazyPerSecondWindows, LazyWindowStartVec, PerBlockFullFromCumulative, shapes::Windows,
};
use brk_error::Result;
use brk_types::{Height, Version};
use statedb::Spends;
use vecdb::{LazyVec, ReadableCloneableVec};

use super::{STORAGE, TypesVecs, Vecs};

/// Removes the one coinbase input of every block through `height`.
pub(crate) fn without_coinbase(height: Height, total: Count) -> Count {
    total - Count::from(height.incremented())
}

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let db = STORAGE.open_database(context, 20_000_000)?;
        let version = STORAGE.schema_version();

        let count_version = version + Version::ONE;
        let count = PerBlockFullFromCumulative::import(
            &db,
            "input_count",
            count_version,
            &LazyVec::new(
                "input_count_cumulative_source",
                count_version,
                mappings.input_count_source().read_only_boxed_clone(),
                without_coinbase,
            ),
            mappings,
            window_starts,
        )?;
        let per_second =
            LazyPerSecondWindows::new("inputs_per_second", version, &count.rolling.sum);
        let types = TypesVecs::import(
            &db,
            version,
            mappings,
            window_starts,
            &count.cumulative.height,
        )?;

        let this = Self {
            spends: Spends::open(db.path())?,
            db,
            count,
            per_second,
            types,
        };
        STORAGE.finalize_database(&this.db)?;
        Ok(this)
    }
}
