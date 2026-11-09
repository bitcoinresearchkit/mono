use bitview_plugin::ImportContext;
use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_vecs::{LazyWindowStartVec, shapes::Windows};
use brk_error::Result;

use super::{STORAGE, Vecs, hash, rewards};

impl Vecs {
    pub fn import(
        context: ImportContext<'_>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        STORAGE.import(context, 1_000_000, |db, version| {
            let rewards = rewards::Vecs::import(&db, version, mappings, window_starts)?;
            let hash = hash::Vecs::import(&db, version, mappings)?;

            let this = Self {
                db,
                rewards,
                hash,
            };
            Ok(this)
        })
    }
}
