use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{CachedVec, Database, EagerVec, ImportableVec, PcoVec, Rw, StorageMode};

use crate::{NumericValue, Resolutions};

/// Like [`PerBlock`](super::PerBlock) but with height wrapped in [`CachedVec`]
/// for fast repeated reads.
#[derive(Deref, DerefMut, Traversable)]
#[traversable(merge)]
pub struct CachedPerBlock<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
{
    pub height: CachedVec<M::Stored<EagerVec<PcoVec<Height, T>>>>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub resolutions: Box<Resolutions<T>>,
}

impl<T> CachedPerBlock<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        let height = CachedVec::wrap(EagerVec::forced_import(db, name, version)?);

        let resolutions = Resolutions::from_boxed_height_source(
            name,
            height.read_only_boxed_clone(),
            version,
            indexes,
        );

        Ok(Self {
            height,
            resolutions: Box::new(resolutions),
        })
    }
}
