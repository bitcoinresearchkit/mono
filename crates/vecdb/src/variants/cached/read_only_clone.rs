use crate::{ReadOnlyClone, StoredVec};

use super::CachedVec;

impl<V: StoredVec> ReadOnlyClone for CachedVec<V> {
    type ReadOnly = CachedVec<V::ReadOnly>;

    #[inline]
    fn read_only_clone(&self) -> Self::ReadOnly {
        CachedVec {
            inner: self.inner.read_only_clone(),
            cache: self.cache.clone(),
            materialize: self.materialize.clone(),
            budget: self.budget,
            last_access: self.last_access.clone(),
            resident_bytes: self.resident_bytes.clone(),
        }
    }
}
