use std::sync::Arc;

use crate::Version;

use super::{ColumnId, ReadableColumnarVec};

/// Lazy scalar projection of one columnar source.
pub struct LazyColumnVec<S, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
{
    pub(super) name: Arc<str>,
    pub(super) base_version: Version,
    pub(super) source: S,
    pub(super) column: C,
}

impl<S, C> Clone for LazyColumnVec<S, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
{
    fn clone(&self) -> Self {
        Self {
            name: Arc::clone(&self.name),
            base_version: self.base_version,
            source: self.source.clone(),
            column: self.column,
        }
    }
}
