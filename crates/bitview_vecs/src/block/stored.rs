use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, PcoVecValue, Rw, StorageMode};

use crate::{CachedSeries, IndexSources, Resolutions, import_cached};

#[derive(Deref, DerefMut, Traversable)]
#[traversable(merge)]
pub struct PerBlock<T, M: StorageMode = Rw>
where
    T: PcoVecValue + PartialOrd + JsonSchema,
{
    pub height: CachedSeries<Height, T, M>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub resolutions: Box<Resolutions<T>>,
}

impl<T> PerBlock<T>
where
    T: PcoVecValue + PartialOrd + JsonSchema + 'static,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &IndexSources,
    ) -> Result<Self> {
        let height = import_cached(db, name, version)?;

        let resolutions = Resolutions::from_source(name, &height, version, indexes);

        Ok(Self {
            height,
            resolutions: Box::new(resolutions),
        })
    }
}
