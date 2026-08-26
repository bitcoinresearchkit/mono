use std::{collections::BTreeSet, ops::Deref, sync::Arc};

use parking_lot::RwLock;

use crate::{
    AnyVec, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, StoredVec, TypedVec, Version,
    vec_region_name_with,
};

use super::{MutableRawVec, MutableVec};

/// Lean read-only view of a [`MutableVec`](super::MutableVec).
#[derive(Debug, Clone)]
pub struct ReadOnlyMutableVec<V> {
    vec: V,
    holes: Arc<RwLock<Arc<BTreeSet<usize>>>>,
}

impl<V> ReadOnlyMutableVec<V> {
    fn new(vec: V, holes: Arc<RwLock<Arc<BTreeSet<usize>>>>) -> Self {
        Self { vec, holes }
    }
}

impl<V> StoredVec for MutableVec<V>
where
    V: MutableRawVec,
{
    type ReadOnly = ReadOnlyMutableVec<V::ReadOnly>;

    #[inline]
    fn read_only_clone(&self) -> Self::ReadOnly {
        ReadOnlyMutableVec::new(
            self.vec.read_only_clone(),
            Arc::clone(&self.published_holes),
        )
    }
}

impl<V> ReadableCloneableVec<V::I, V::T> for MutableVec<V>
where
    V: MutableRawVec,
{
    #[inline]
    fn read_only_boxed_clone(&self) -> ReadableBoxedVec<V::I, V::T> {
        ReadableBoxedVec::new(self.read_only_clone())
    }
}

impl<V> Deref for ReadOnlyMutableVec<V> {
    type Target = V;

    #[inline]
    fn deref(&self) -> &Self::Target {
        &self.vec
    }
}

impl<V> AnyVec for ReadOnlyMutableVec<V>
where
    V: AnyVec + TypedVec,
{
    #[inline]
    fn version(&self) -> Version {
        self.vec.version()
    }

    #[inline]
    fn name(&self) -> &str {
        self.vec.name()
    }

    #[inline]
    fn len(&self) -> usize {
        self.vec.len()
    }

    #[inline]
    fn is_mutable(&self) -> bool {
        true
    }

    #[inline]
    fn index_type_to_string(&self) -> &'static str {
        self.vec.index_type_to_string()
    }

    fn region_names(&self) -> Vec<String> {
        let mut names = self.vec.region_names();
        if !self.holes.read().is_empty() {
            names.push(format!(
                "{}_holes",
                vec_region_name_with::<V::I>(self.vec.name())
            ));
        }
        names
    }

    #[inline]
    fn value_type_to_size_of(&self) -> usize {
        self.vec.value_type_to_size_of()
    }

    #[inline]
    fn value_type_to_string(&self) -> &'static str {
        self.vec.value_type_to_string()
    }
}

impl<V: TypedVec> TypedVec for ReadOnlyMutableVec<V> {
    type I = V::I;
    type T = V::T;
}

impl<V> ReadableVec<V::I, V::T> for ReadOnlyMutableVec<V>
where
    V: TypedVec + ReadableVec<V::I, V::T>,
{
    #[inline(always)]
    fn cursor_chunk_size(&self) -> usize {
        self.vec.cursor_chunk_size()
    }

    #[inline(always)]
    fn collect_one_at(&self, index: usize) -> Option<V::T> {
        if self.holes.read().contains(&index) {
            None
        } else {
            self.vec.collect_one_at(index)
        }
    }

    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<V::T>) {
        self.fold_range_at(from, to, (), |(), value| buf.push(value));
    }

    #[inline]
    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<V::T>) {
        let holes = self.holes.read();
        if holes.is_empty() {
            self.vec.read_sorted_into_at(indices, out);
            return;
        }

        out.reserve(indices.len());
        for &index in indices {
            if !holes.contains(&index)
                && let Some(value) = self.vec.collect_one_at(index)
            {
                out.push(value);
            }
        }
    }

    #[inline]
    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(V::T)) {
        self.fold_range_at(from, to, (), |(), value| f(value));
    }

    fn fold_range_at<B, F>(&self, from: usize, to: usize, init: B, mut f: F) -> B
    where
        Self: Sized,
        F: FnMut(B, V::T) -> B,
    {
        let len = self.vec.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return init;
        }

        let holes = self.holes.read();
        if holes.is_empty() {
            return self.vec.fold_range_at(from, to, init, f);
        }

        let mut start = from;
        let mut acc = init;
        for &hole in holes.range(from..to) {
            if start < hole {
                acc = self.vec.fold_range_at(start, hole, acc, &mut f);
            }
            start = hole + 1;
        }
        self.vec.fold_range_at(start, to, acc, f)
    }

    fn try_fold_range_at<B, E, F>(&self, from: usize, to: usize, init: B, mut f: F) -> Result<B, E>
    where
        Self: Sized,
        F: FnMut(B, V::T) -> Result<B, E>,
    {
        let len = self.vec.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return Ok(init);
        }

        let holes = self.holes.read();
        if holes.is_empty() {
            return self.vec.try_fold_range_at(from, to, init, f);
        }

        let mut start = from;
        let mut acc = init;
        for &hole in holes.range(from..to) {
            if start < hole {
                acc = self.vec.try_fold_range_at(start, hole, acc, &mut f)?;
            }
            start = hole + 1;
        }
        self.vec.try_fold_range_at(start, to, acc, f)
    }
}
