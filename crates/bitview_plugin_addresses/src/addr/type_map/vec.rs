use bitview_primitives::ByAddrType;
use derive_more::{Deref, DerefMut};

/// A vector for each address type.
#[derive(Debug, Deref, DerefMut)]
pub struct AddrTypeToVec<T>(ByAddrType<Vec<T>>);

impl<T> Default for AddrTypeToVec<T> {
    fn default() -> Self {
        Self(ByAddrType::default())
    }
}

impl<T> AddrTypeToVec<T> {
    /// Consume the wrapper and return its vectors.
    pub fn into_inner(self) -> ByAddrType<Vec<T>> {
        self.0
    }
}
