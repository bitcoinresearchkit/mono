use brk_error::Result;
use brk_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, Rw, StorageMode};

use crate::{
    indexes,
    internal::{ColumnarPerBlock, LazyColumnPerBlock, NumericValue, WindowFrom1wId, WindowsFrom1w},
};

#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct ColumnarRollingWindowsFrom1w<T, M: StorageMode = Rw>(
    pub ColumnarPerBlock<T, WindowFrom1wId, WindowsFrom1w<LazyColumnPerBlock<T, WindowFrom1wId>>, M>,
)
where
    T: NumericValue + JsonSchema;

impl<T> ColumnarRollingWindowsFrom1w<T>
where
    T: NumericValue + JsonSchema,
{
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        Ok(Self(ColumnarPerBlock::forced_import(
            db,
            name,
            version,
            |source| {
                WindowFrom1wId::series(|window| {
                    LazyColumnPerBlock::new(
                        &format!("{name}_{}", window.suffix()),
                        version,
                        source,
                        window,
                        indexes,
                    )
                })
            },
        )?))
    }
}
