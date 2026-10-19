mod convert;
pub mod ident;
mod map_option;

pub use convert::Convert;
pub use ident::Ident;
pub use map_option::MapOption;

use crate::{READ_CHUNK_SIZE, ReadableBoxedVec, ReadableVec, VecIndex, VecValue};

/// A unary operation passed by type (usually a zero-sized struct): calls are statically
/// dispatched.
pub trait UnaryTransform<In, Out = In> {
    fn apply(value: In) -> Out;

    /// Bulk transform selected once per read, keeping the element loop static.
    fn read_into<I: VecIndex>(
        source: &ReadableBoxedVec<I, In>,
        from: usize,
        to: usize,
        out: &mut Vec<Out>,
    ) where
        In: VecValue,
        Out: VecValue,
    {
        source.for_each_chunk_at(from, to, &mut |_, values| {
            out.extend(values.iter().cloned().map(Self::apply));
        });
    }

    /// Borrows each transformed chunk; identity transforms can forward the
    /// original slice without allocating a result buffer.
    fn for_each_chunk<I: VecIndex>(
        source: &ReadableBoxedVec<I, In>,
        from: usize,
        to: usize,
        f: &mut dyn FnMut(usize, &[Out]),
    ) where
        In: VecValue,
        Out: VecValue,
    {
        for_each_mapped_chunk(source, from, to, f, |_, value| Self::apply(value));
    }
}

/// Visits `source`'s values in `[from, to)` mapped by `map(index, value)`, through one reused
/// buffer in chunks of at most the source's cursor chunk size (capped at [`READ_CHUNK_SIZE`]).
pub(super) fn for_each_mapped_chunk<I: VecIndex, In: VecValue, Out>(
    source: &ReadableBoxedVec<I, In>,
    from: usize,
    to: usize,
    f: &mut dyn FnMut(usize, &[Out]),
    mut map: impl FnMut(usize, In) -> Out,
) {
    let mut output = Vec::new();
    let chunk_size = source.cursor_chunk_size().clamp(1, READ_CHUNK_SIZE);
    source.for_each_chunk_at(from, to, &mut |mut at, values| {
        for values in values.chunks(chunk_size) {
            output.clear();
            output.extend(
                values
                    .iter()
                    .cloned()
                    .enumerate()
                    .map(|(offset, value)| map(at + offset, value)),
            );
            f(at, &output);
            at += values.len();
        }
    });
}
