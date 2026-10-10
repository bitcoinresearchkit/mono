use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_traversable::Traversable;
use bitview_vecs::{CachedSeries, LazySpotValuePerBlock};
use brk_error::Result;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{AnyStoredVec, Database, ReadableBoxedVec, Rw, StorageMode, WritableVec};

use super::import_stored;

/// One cohort's sats, stored as `{name}_sats` and valued at the block's spot price.
#[derive(Traversable)]
pub struct Supply<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub value: LazySpotValuePerBlock,
    #[traversable(hidden)]
    pub stored: CachedSeries<Height, Sats, M>,
}

impl Supply {
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &MappingsVecs,
        spot_price: &ReadableBoxedVec<Height, Cents>,
    ) -> Result<Self> {
        let stored = import_stored(db, &format!("{name}_sats"), version)?;
        let value =
            LazySpotValuePerBlock::from_sats_source(name, version, &stored, mappings, spot_price);
        Ok(Self { value, stored })
    }

    #[inline(always)]
    pub fn push(&mut self, sats: Sats) {
        self.stored.push(sats);
    }

    pub fn stored_mut(&mut self) -> &mut dyn AnyStoredVec {
        &mut self.stored
    }
}
