use brk_error::Result;

use bitview_traversable::Traversable;
use brk_exit::Exit;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{
    BinaryTransform, Database, EagerVec, ImportableVec, PcoVec, PcoVecValue, ReadOnlyClone,
    ReadableVec, Rw, StorageMode, VecValue,
};

use crate::Resolutions;

#[derive(Deref, DerefMut, Traversable)]
#[traversable(merge)]
pub struct PerBlock<T, M: StorageMode = Rw>
where
    T: PcoVecValue + PartialOrd + JsonSchema,
{
    pub height: M::Stored<EagerVec<PcoVec<Height, T>>>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub resolutions: Box<Resolutions<T>>,
}

impl<T> PerBlock<T>
where
    T: PcoVecValue + PartialOrd + JsonSchema + 'static,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        let height: EagerVec<PcoVec<Height, T>> = EagerVec::forced_import(db, name, version)?;

        let resolutions =
            Resolutions::from_height_source(name, height.read_only_clone(), version, indexes);

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
