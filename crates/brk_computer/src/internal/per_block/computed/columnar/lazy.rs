use brk_traversable::Traversable;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    ColumnId, Formattable, LazyColumnVec, PcoVec, PcoVecValue, ReadOnlyColumnarVec,
    ReadableColumnarVec,
};

use crate::{indexes, internal::Resolutions};

#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(merge)]
pub struct LazyColumnPerBlock<T, C>
where
    T: PcoVecValue + Formattable + PartialOrd + Serialize + JsonSchema,
    C: ColumnId,
{
    pub height: LazyColumnVec<ReadOnlyColumnarVec<PcoVec<Height, T>, C>, C>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub resolutions: Box<Resolutions<T>>,
}

impl<T, C> LazyColumnPerBlock<T, C>
where
    T: PcoVecValue + Formattable + PartialOrd + Serialize + JsonSchema + 'static,
    C: ColumnId,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, T>, C>,
        column: C,
        indexes: &indexes::Vecs,
    ) -> Self {
        let height = source.column(name, version, column);
        let resolutions = Resolutions::forced_import(name, height.clone(), version, indexes);

        Self {
            height,
            resolutions: Box::new(resolutions),
        }
    }
}
