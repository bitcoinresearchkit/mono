use brk_error::Result;

use bitview_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use vecdb::{Database, Rw, StorageMode};

use crate::{ColumnarPerBlock, FixedRatio, LazyColumnPercentPerBlock, WindowId, Windows};

/// Four named fixed-point percentage views backed by one columnar source.
#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct ColumnarPercentRollingWindows<B: FixedRatio, M: StorageMode = Rw>(
    pub ColumnarPerBlock<B, WindowId, Windows<LazyColumnPercentPerBlock<B, WindowId>>, M>,
);

impl<B: FixedRatio> ColumnarPercentRollingWindows<B> {
    pub fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &crate::IndexSources,
    ) -> Result<Self> {
        Ok(Self(ColumnarPerBlock::forced_import(
            db,
            &format!("{name}_{}", B::SUFFIX),
            version,
            |source| {
                WindowId::series(|window| {
                    LazyColumnPercentPerBlock::new(
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
