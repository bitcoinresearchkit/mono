use std::sync::Arc;

pub mod any_vec;
pub mod read_only_clone;
pub mod readable;
pub mod transform;
pub mod typed;

pub use transform::*;

use crate::{ReadableBoxedVec, ReadableVec, VecIndex, VecValue, Version};

/// Lazily computed vector deriving values on-the-fly from one source vector.
///
/// Unlike a stored vec computed with `ComputeVec`, no data is stored on disk. Values are
/// computed during iteration by applying a function to the source vector's elements. Use when:
/// - Storage space is limited
/// - Computation is cheap relative to disk I/O
/// - Values are only accessed once or infrequently
///
/// For frequently accessed derived data, prefer a stored vec computed with `ComputeVec`.
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
    pub fn init(
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

    /// Create a lazy vec with a generic transform.
    /// Usage: `LazyVec::transformed::<Ident>(name, v, source)`
    pub fn transformed<F: UnaryTransform<S, T>>(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<I, S>,
    ) -> Self {
        let mut vec = Self::init(name, version, source, |_, v| F::apply(v));
        vec.read = |this, from, to, out| F::read_into(&this.source, from, to, out);
        vec.visit = |this, from, to, f| F::for_each_chunk(&this.source, from, to, f);
        vec
    }
}
