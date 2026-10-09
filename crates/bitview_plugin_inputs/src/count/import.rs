use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::Count;
use bitview_vecs::{LazyWindowStartVec, PerBlockFullFromCumulative};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, LazyVec, ReadableCloneableVec};

use super::Vecs;

/// Removes the one coinbase input of every block through `height`.
pub(crate) fn without_coinbase(height: Height, total: Count) -> Count {
    total - Count::from(height.incremented())
}

impl Vecs {
    pub(crate) fn import(
        db: &Database,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let cumulative = LazyVec::init(
            "input_count_cumulative_source",
            version,
            mappings.input_count_source().read_only_boxed_clone(),
            without_coinbase,
        );
        Ok(Self(PerBlockFullFromCumulative::import(
            db,
            "input_count",
            version,
            &cumulative,
            mappings,
            window_starts,
        )?))
    }
}
