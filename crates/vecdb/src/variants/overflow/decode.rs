use super::DECODE_CHUNK_SIZE;
use crate::{OverflowVecValue, ReadableVec, VecIndex, unlikely};

/// Decode the representation independently of the reader's visibility policy.
#[inline(always)]
pub(super) fn decode<T: OverflowVecValue>(
    compact: T::Compact,
    overflow: impl FnOnce(usize) -> T,
) -> T {
    let index = T::overflow_index(compact);
    if unlikely(index.is_some()) {
        overflow(index.unwrap())
    } else {
        T::from_compact(compact)
    }
}

/// Appends the values `[from, to)` below `len`, decoding overflow pointers through the lookup
/// `sidecar` opens (only for a non-empty range): long ranges through a bounded buffer of compact
/// values.
pub(super) fn read_into<I: VecIndex, T: OverflowVecValue, F: Fn(usize) -> T>(
    compact: &impl ReadableVec<I, T::Compact>,
    len: usize,
    from: usize,
    to: usize,
    buf: &mut Vec<T>,
    sidecar: impl FnOnce() -> F,
) {
    let from = from.min(len);
    let to = to.min(len);
    if from >= to {
        return;
    }
    let overflow = sidecar();
    buf.reserve(to - from);
    if to - from > DECODE_CHUNK_SIZE {
        let mut values = Vec::with_capacity(DECODE_CHUNK_SIZE);
        for chunk_from in (from..to).step_by(DECODE_CHUNK_SIZE) {
            values.clear();
            compact.read_into_at(
                chunk_from,
                (chunk_from + DECODE_CHUNK_SIZE).min(to),
                &mut values,
            );
            buf.extend(values.iter().map(|&value| decode(value, &overflow)));
        }
        return;
    }
    compact.for_each_range_at(from, to, |value| buf.push(decode(value, &overflow)));
}

/// Appends the values at sorted `indices` below `len`, decoding overflow pointers through the
/// lookup `sidecar` opens once the compact values are read: a contiguous run reads as one range.
pub(super) fn read_sorted_into<I: VecIndex, T: OverflowVecValue, F: Fn(usize) -> T>(
    compact: &impl ReadableVec<I, T::Compact>,
    len: usize,
    indices: &[usize],
    out: &mut Vec<T>,
    sidecar: impl FnOnce() -> F,
) {
    let indices = &indices[..indices.partition_point(|&i| i < len)];
    let (Some(&first), Some(&last)) = (indices.first(), indices.last()) else {
        return;
    };
    let values = if indices.len() > 1 && indices.windows(2).all(|pair| pair[1] == pair[0] + 1) {
        compact.collect_range_at(first, last + 1)
    } else {
        compact.read_sorted_at(indices)
    };
    let overflow = sidecar();
    out.extend(values.into_iter().map(|value| decode(value, &overflow)));
}
