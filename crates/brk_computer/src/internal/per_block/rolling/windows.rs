//! RollingWindows - newtype on Windows with PerBlock per window duration.
//!
//! Each of the 4 windows (24h, 1w, 1m, 1y) contains a height-level vec plus
//! all 17 LazyAggVec index views.

use brk_error::Result;

use brk_traversable::Traversable;
use brk_types::Version;
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{Database, Rw, StorageMode};

use crate::{
    blocks::lookback::LazyWindowStartVec,
    indexes,
    internal::{
        ColumnarPerBlock, ComputedVecValue, LazyColumnPerBlock, NumericValue, PerBlock,
        WindowFrom1wId, WindowId, Windows, WindowsFrom1w,
    },
};

pub use crate::blocks::lookback::CachedWindowStartVec;

/// Rolling window start heights — the 4 height-ago vecs (24h, 1w, 1m, 1y).
#[derive(Deref, DerefMut)]
pub struct WindowStarts<'a>(pub Windows<&'a LazyWindowStartVec>);

/// 4 rolling window vecs (24h, 1w, 1m, 1y), each with height data + all 17 index views.
#[derive(Deref, DerefMut, Traversable)]
#[traversable(transparent)]
pub struct RollingWindows<T, M: StorageMode = Rw>(pub Windows<PerBlock<T, M>>)
where
    T: ComputedVecValue + PartialOrd + JsonSchema;

impl<T> RollingWindows<T>
where
    T: NumericValue + JsonSchema,
{
    pub(crate) fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        Ok(Self(Windows::try_from_fn(|suffix| {
            PerBlock::forced_import(db, &format!("{name}_{suffix}"), version, indexes)
        })?))
    }
}

/// Four named rolling-window views backed by one columnar source.
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

/// The 1w, 1m, and 1y views backed by one columnar source.
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
