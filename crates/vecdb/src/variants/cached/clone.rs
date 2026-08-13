use crate::TypedVec;

use super::CachedVec;

impl<V: TypedVec + Clone> Clone for CachedVec<V> {
    #[inline(always)]
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            cache: self.cache.clone(),
            materialize: self.materialize.clone(),
            budget: self.budget,
            access_count: self.access_count.clone(),
            last_access: self.last_access.clone(),
            resident_bytes: self.resident_bytes.clone(),
        }
    }
}
