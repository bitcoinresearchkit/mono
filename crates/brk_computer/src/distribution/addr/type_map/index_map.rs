use std::{collections::hash_map::Entry, mem};

use brk_cohort::ByAddrType;
use brk_types::{OutputType, TypeIndex};
use derive_more::{Deref, DerefMut};
use rustc_hash::FxHashMap;
use smallvec::{Array, SmallVec};

/// A hashmap for each address type, keyed by TypeIndex.
#[derive(Debug, Deref, DerefMut)]
pub struct AddrTypeToTypeIndexMap<T>(ByAddrType<FxHashMap<TypeIndex, T>>);

impl<T> Default for AddrTypeToTypeIndexMap<T> {
    fn default() -> Self {
        Self(ByAddrType {
            p2a: FxHashMap::default(),
            p2pk33: FxHashMap::default(),
            p2pk65: FxHashMap::default(),
            p2pkh: FxHashMap::default(),
            p2sh: FxHashMap::default(),
            p2tr: FxHashMap::default(),
            p2wpkh: FxHashMap::default(),
            p2wsh: FxHashMap::default(),
        })
    }
}

impl<T> AddrTypeToTypeIndexMap<T> {
    /// Create with pre-allocated capacity per address type.
    pub fn with_capacity(capacity: usize) -> Self {
        Self(ByAddrType {
            p2a: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            p2pk33: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            p2pk65: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            p2pkh: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            p2sh: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            p2tr: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            p2wpkh: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
            p2wsh: FxHashMap::with_capacity_and_hasher(capacity, Default::default()),
        })
    }

    /// Insert a value for a specific address type and type_index.
    pub fn insert_for_type(&mut self, addr_type: OutputType, type_index: TypeIndex, value: T) {
        self.get_mut(addr_type).unwrap().insert(type_index, value);
    }

    /// Consume and iterate over entries by address type.
    #[allow(clippy::should_implement_trait)]
    pub fn into_iter(self) -> impl Iterator<Item = (OutputType, FxHashMap<TypeIndex, T>)> {
        self.0.into_iter()
    }

    /// Consume and return the inner ByAddrType.
    pub fn into_inner(self) -> ByAddrType<FxHashMap<TypeIndex, T>> {
        self.0
    }

    /// Iterate mutably over entries by address type.
    pub fn iter_mut(&mut self) -> impl Iterator<Item = (OutputType, &mut FxHashMap<TypeIndex, T>)> {
        self.0.iter_mut()
    }
}

impl<T> AddrTypeToTypeIndexMap<SmallVec<T>>
where
    T: Array,
{
    /// Merge two maps of SmallVec values, concatenating vectors.
    pub fn merge_vec(mut self, other: Self) -> Self {
        for (addr_type, other_map) in other.0.into_iter() {
            let self_map = self.0.get_mut_unwrap(addr_type);
            for (type_index, mut other_vec) in other_map {
                match self_map.entry(type_index) {
                    Entry::Occupied(mut entry) => {
                        let self_vec = entry.get_mut();
                        if other_vec.len() > self_vec.len() {
                            mem::swap(self_vec, &mut other_vec);
                        }
                        self_vec.extend(other_vec);
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(other_vec);
                    }
                }
            }
        }
        self
    }
}
