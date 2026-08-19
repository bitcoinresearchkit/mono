use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, Rw, StorageMode};

use crate::{ColumnarPerBlock, LazyColumnPerBlock, NumericValue, WindowId, Windows};

#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct ColumnarRollingWindows<T, M: StorageMode = Rw>(
    pub ColumnarPerBlock<T, WindowId, Windows<LazyColumnPerBlock<T, WindowId>>, M>,
)
where
    T: NumericValue + JsonSchema;

impl<T> ColumnarRollingWindows<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        Ok(Self(ColumnarPerBlock::forced_import(
            db,
            name,
            version,
            |source| {
                WindowId::series(|window| {
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
