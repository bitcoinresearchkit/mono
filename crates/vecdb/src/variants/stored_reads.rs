//! Reads of the raw and compressed stored vecs and their read-only clones: the retained cache
//! when there is one, else the format's persisted values below the stored length, then the
//! pushed values after it (none for read-only clones).

use std::{ops::Range, result::Result};

use crate::{
    READ_CHUNK_SIZE, ReadableVec, VecIndex, VecValue,
    cache::{Cache, Request},
    traits::chunk_folds,
};

/// A format's persisted values. Every read is bounded by `stored`, then again by what is on
/// disk: a truncation may land after a caller's length.
pub(crate) trait PersistedReads<T: VecValue> {
    /// Decodes the ranges missing from a cache, each into its own buffer.
    fn load(&self, stored: usize, missing: &[Range<usize>]) -> Vec<(usize, Vec<T>)>;

    fn read(&self, stored: usize, from: usize, to: usize, out: &mut Vec<T>);

    /// The value at `index`, below `stored`.
    fn collect_one(&self, stored: usize, index: usize) -> Option<T>;

    /// The values at sorted `indices`, all below `stored`.
    fn read_sorted(&self, stored: usize, indices: &[usize], out: &mut Vec<T>);

    /// Lends `[from, to)` in chunks; false, visiting nothing, when only a buffered read can.
    fn for_each_chunk(
        &self,
        stored: usize,
        from: usize,
        to: usize,
        f: &mut dyn FnMut(usize, &[T]),
    ) -> bool;

    fn try_fold<B, E, F: FnMut(B, T) -> Result<B, E>>(
        &self,
        stored: usize,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E>;
}

pub(crate) trait StoredReads<T: VecValue> {
    type Persisted<'a>: PersistedReads<T>
    where
        Self: 'a;

    /// Values per cursor refill (see [`ReadableVec::cursor_chunk_size`]).
    const CHUNK_SIZE: usize = READ_CHUNK_SIZE;
    /// Pushed values lent per chunk.
    const PUSHED_CHUNK: usize = usize::MAX;

    fn cache(&self) -> Option<&Cache<T>>;

    /// The stored length and the pushed values after it.
    fn layers(&self) -> (usize, &[T]);

    fn persisted(&self) -> Self::Persisted<'_>;
}

pub(crate) fn read_cached<T: VecValue>(
    v: &impl StoredReads<T>,
    from: usize,
    to: usize,
    out: &mut Vec<T>,
) -> bool {
    v.cache()
        .is_some_and(|cache| cache.try_read_range(from, to, || v.layers().0, out))
}

#[inline(always)]
pub(crate) fn collect_one<T: VecValue>(v: &impl StoredReads<T>, index: usize) -> Option<T> {
    if let Some(cache) = v.cache() {
        return cache.get_source(
            index,
            || v.layers(),
            |stored, missing| v.persisted().load(stored, missing),
        );
    }
    let (stored, pushed) = v.layers();
    if index >= stored {
        return pushed.get(index - stored).cloned();
    }
    v.persisted().collect_one(stored, index)
}

#[inline(always)]
pub(crate) fn read_into<T: VecValue>(
    v: &impl StoredReads<T>,
    from: usize,
    to: usize,
    buf: &mut Vec<T>,
) {
    if let Some(cache) = v.cache() {
        return cache.read_source(
            Request::Range(from, to),
            buf,
            || v.layers(),
            |stored, missing| v.persisted().load(stored, missing),
        );
    }
    let (stored, pushed) = v.layers();
    let len = stored + pushed.len();
    let from = from.min(len);
    let to = to.min(len);
    if from >= to {
        return;
    }
    buf.reserve(to - from);
    v.persisted().read(stored, from, to, buf);
    if to > stored {
        buf.extend_from_slice(&pushed[from.max(stored) - stored..to - stored]);
    }
}

#[inline]
pub(crate) fn read_sorted<T: VecValue>(
    v: &impl StoredReads<T>,
    indices: &[usize],
    out: &mut Vec<T>,
) {
    if let Some(cache) = v.cache() {
        return cache.read_source(
            Request::Sorted(indices),
            out,
            || v.layers(),
            |stored, missing| v.persisted().load(stored, missing),
        );
    }
    let (stored, pushed) = v.layers();
    let indices = &indices[..indices.partition_point(|&i| i < stored + pushed.len())];
    let split = indices.partition_point(|&i| i < stored);
    out.reserve(indices.len());
    v.persisted().read_sorted(stored, &indices[..split], out);
    out.extend(indices[split..].iter().map(|&i| pushed[i - stored].clone()));
}

pub(crate) fn for_each_chunk<I: VecIndex, T: VecValue, V>(
    v: &V,
    from: usize,
    to: usize,
    f: &mut dyn FnMut(usize, &[T]),
) where
    V: StoredReads<T> + ReadableVec<I, T>,
{
    if let Some(cache) = v.cache() {
        return cache.for_each_source(
            from,
            to,
            || v.layers(),
            |stored, missing| v.persisted().load(stored, missing),
            f,
        );
    }
    let (stored, pushed) = v.layers();
    let to = to.min(stored + pushed.len());
    if from >= to {
        return;
    }
    if !v.persisted().for_each_chunk(stored, from, to, f) {
        return chunk_folds::for_each_chunk(v, from, to, f);
    }
    if to > stored {
        let mut at = from.max(stored);
        for values in pushed[at - stored..to - stored].chunks(V::PUSHED_CHUNK) {
            f(at, values);
            at += values.len();
        }
    }
}

#[inline]
pub(crate) fn try_fold<T: VecValue, B, E, F: FnMut(B, T) -> Result<B, E>>(
    v: &impl StoredReads<T>,
    from: usize,
    to: usize,
    init: B,
    mut f: F,
) -> Result<B, E> {
    if let Some(cache) = v.cache() {
        return cache.try_fold_source(
            from,
            to,
            init,
            || v.layers(),
            |stored, missing| v.persisted().load(stored, missing),
            f,
        );
    }
    let (stored, pushed) = v.layers();
    let len = stored + pushed.len();
    let from = from.min(len);
    let to = to.min(len);
    if from >= to {
        return Ok(init);
    }
    if to <= stored {
        return v.persisted().try_fold(stored, from, to, init, f);
    }
    let mut acc = init;
    if from < stored {
        acc = v.persisted().try_fold(stored, from, stored, acc, &mut f)?;
    }
    pushed[from.max(stored) - stored..to - stored]
        .iter()
        .cloned()
        .try_fold(acc, f)
}
