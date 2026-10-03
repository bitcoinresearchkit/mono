use bitview_traversable::Traversable;
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{BinaryTransform, Database, PcoVecValue, ReadableVec, Rw, StorageMode, VecValue};

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

    /// Eagerly compute this vec as a binary transform of two sources.
    pub fn compute_binary<S1T, S2T, F>(
        &mut self,
        max_from: Height,
        source1: &impl ReadableVec<Height, S1T>,
        source2: &impl ReadableVec<Height, S2T>,
        exit: &Exit,
    ) -> Result<()>
    where
        S1T: VecValue,
        S2T: VecValue,
        F: BinaryTransform<S1T, S2T, T>,
    {
        self.height
            .compute_binary::<S1T, S2T, F>(max_from, source1, source2, exit)?;
        Ok(())
    }
}
