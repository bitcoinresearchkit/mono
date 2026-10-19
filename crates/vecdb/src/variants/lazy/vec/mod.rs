use std::sync::Arc;

pub mod any_vec;
pub mod read_only_clone;
pub mod readable;
pub mod transform;
pub mod typed;

pub use transform::*;

use crate::{ReadableBoxedVec, ReadableVec, VecIndex, VecValue, Version};

/// A view mapping each value of one source vector on read; nothing is stored on disk. Persist
/// with `ComputeVec` instead when recomputing costs more than reading.
///
/// The source must be dense: every index below its length holds a value, so each value pairs
/// with its index. Reads cannot check it: a source may grow or be truncated while read.
#[derive(Clone)]
pub struct LazyVec<I, T, S>
where
    I: VecIndex,
    S: VecValue,
{
    name: Arc<str>,
    base_version: Version,
    source: ReadableBoxedVec<I, S>,
    compute: fn(I, S) -> T,
    read: fn(&Self, usize, usize, &mut Vec<T>),
    visit: fn(&Self, usize, usize, &mut dyn FnMut(usize, &[T])),
}

impl<I, T, S> LazyVec<I, T, S>
where
    I: VecIndex,
    T: VecValue,
    S: VecValue,
{
    pub fn new(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<I, S>,
        compute: fn(I, S) -> T,
    ) -> Self {
        Self {
            name: Arc::from(name),
            base_version: version,
            source,
            compute,
            read: |this, from, to, out| {
                this.source.for_each_chunk_at(from, to, &mut |at, values| {
                    out.extend(
                        values
                            .iter()
                            .cloned()
                            .enumerate()
                            .map(|(offset, value)| (this.compute)(I::from(at + offset), value)),
                    );
                });
            },
            visit: |this, from, to, f| {
                transform::for_each_mapped_chunk(&this.source, from, to, f, |index, value| {
                    (this.compute)(I::from(index), value)
                });
            },
        }
    }

    pub fn transformed<F: UnaryTransform<S, T>>(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<I, S>,
    ) -> Self {
        let mut vec = Self::new(name, version, source, |_, v| F::apply(v));
        vec.read = |this, from, to, out| F::read_into(&this.source, from, to, out);
        vec.visit = |this, from, to, f| F::for_each_chunk(&this.source, from, to, f);
        vec
    }
}
