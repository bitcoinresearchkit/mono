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
    pub(super) vec: V::ReadOnly,
    pub(super) visible_rows: SharedLen,
    pub(super) gate: Arc<RwLock<()>>,
    pub(super) columns: PhantomData<C>,
}

impl<V, C> Clone for ReadOnlyColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    fn clone(&self) -> Self {
        Self {
            vec: self.vec.clone(),
            visible_rows: self.visible_rows.clone(),
            gate: Arc::clone(&self.gate),
            columns: PhantomData,
        }
    }
}
