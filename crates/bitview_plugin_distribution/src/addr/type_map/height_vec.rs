use brk_types::Height;
use derive_more::{Deref, DerefMut};
use rustc_hash::FxHashMap;

use super::vec::AddrTypeToVec;

/// Hashmap from Height to AddrTypeToVec.
#[derive(Debug, Default, Deref, DerefMut)]
pub struct HeightToAddrTypeToVec<T>(FxHashMap<Height, AddrTypeToVec<T>>);

impl<T> HeightToAddrTypeToVec<T> {
    /// Create with pre-allocated capacity for unique heights.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(FxHashMap::with_capacity_and_hasher(
            capacity,
            Default::default(),
        ))
    }
}

impl<T> HeightToAddrTypeToVec<T> {
    /// Consume and iterate over (Height, AddrTypeToVec) pairs.
    pub fn into_iter(self) -> impl Iterator<Item = (Height, AddrTypeToVec<T>)> {
        self.0.into_iter()
    }
}
