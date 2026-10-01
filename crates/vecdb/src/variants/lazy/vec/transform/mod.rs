pub mod ident;
mod map_option;

pub use ident::Ident;
pub use map_option::MapOption;

use crate::{READ_CHUNK_SIZE, ReadableBoxedVec, ReadableVec, VecIndex, VecValue};

/// Trait for unary transforms applied lazily during iteration.
/// Zero-sized types implementing this get monomorphized (zero runtime cost).
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
        let mut output = Vec::new();
        let chunk_size = source.cursor_chunk_size().clamp(1, READ_CHUNK_SIZE);
        source.for_each_chunk_at(from, to, &mut |at, values| {
            let mut at = at;
            for values in values.chunks(chunk_size) {
                output.clear();
                output.extend(values.iter().cloned().map(Self::apply));
                f(at, &output);
                at += values.len();
            }
        });
    }
}
