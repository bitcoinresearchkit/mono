use std::{marker::PhantomData, sync::Arc};

use parking_lot::RwLock;

use crate::{SharedLen, StoredVec};

use super::ColumnId;

/// Lean read-only clone of a columnar vector.
pub struct ReadOnlyColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    pub(super) name: Arc<str>,
    pub(super) columns: Arc<[V::ReadOnly]>,
    pub(super) visible_rows: SharedLen,
    pub(super) gate: Arc<RwLock<()>>,
    pub(super) column_ids: PhantomData<C>,
}

impl<V, C> Clone for ReadOnlyColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    fn clone(&self) -> Self {
        Self {
            name: Arc::clone(&self.name),
            columns: Arc::clone(&self.columns),
            visible_rows: self.visible_rows.clone(),
            gate: Arc::clone(&self.gate),
            column_ids: PhantomData,
        }
    }
}
