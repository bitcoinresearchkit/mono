use bitview_collections::Windows;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Count, CountSigned, PartsPerMillionSigned64};
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazyPerBlockWithDeltas, LazyWindowStartVec};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{AnyStoredVec, Database, Rw, StorageMode, WritableVec};

use super::import_stored;

/// A count, stored under its name, with its change over each trailing window.
#[derive(Traversable)]
pub struct CountWithDeltas<M: StorageMode = Rw> {
    #[traversable(flatten)]
    count: LazyPerBlockWithDeltas<Count, CountSigned, PartsPerMillionSigned64>,
    #[traversable(hidden)]
    pub stored: CachedSeries<Height, Count, M>,
}

impl CountWithDeltas {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        let stored = import_stored(db, name, version)?;
        let count = LazyPerBlockWithDeltas::from_height_source(
            name,
            version,
            &stored,
            Version::TWO,
            mappings,
            window_starts,
        );
        Ok(Self { count, stored })
    }

    #[inline(always)]
    pub fn push(&mut self, count: Count) {
        self.stored.push(count);
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.stored
    }
}
