use crate::{ReadableVec, VecIndex, VecValue};

/// End of the chunk read from `at`: the next multiple of `chunk_size` (so every read after the
/// first starts on a source page boundary), capped at `to`.
#[inline]
pub(crate) fn chunk_end(at: usize, chunk_size: usize, to: usize) -> usize {
    at.saturating_add(chunk_size - at % chunk_size).min(to)
}

pub(crate) fn for_each_chunk<I: VecIndex, T: VecValue>(
    source: &(impl ReadableVec<I, T> + ?Sized),
    from: usize,
    to: usize,
    each: &mut dyn FnMut(usize, &[T]),
) {
    let to = to.min(source.len());
    let chunk_size = source.cursor_chunk_size().max(1);
    let mut values = Vec::with_capacity(chunk_size.min(to.saturating_sub(from)));
    let mut at = from;
    let mut emitted_at = from;
    while at < to {
        let end = chunk_end(at, chunk_size, to);
        values.clear();
        source.read_into_at(at, end, &mut values);
        if !values.is_empty() {
            each(emitted_at, &values);
            emitted_at += values.len();
        }
        at = end;
    }
}

/// Folds borrowed chunks without routing each value through a trait object.
#[inline]
pub(crate) fn fold<I: VecIndex, T: VecValue, B>(
    source: &(impl ReadableVec<I, T> + ?Sized),
    from: usize,
    to: usize,
    init: B,
    mut f: impl FnMut(B, T) -> B,
) -> B {
    let mut acc = Some(init);
    source.for_each_chunk_at(from, to, &mut |_, values| {
        acc = Some(values.iter().cloned().fold(acc.take().unwrap(), &mut f));
    });
    acc.unwrap()
}

/// Buffered fallback for erased sources: stop before reading another chunk
/// after the first error. Borrowed chunk callbacks cannot return early.
#[inline]
pub(crate) fn try_fold<I: VecIndex, T: VecValue, B, E>(
    source: &(impl ReadableVec<I, T> + ?Sized),
    from: usize,
    to: usize,
    init: B,
    mut f: impl FnMut(B, T) -> Result<B, E>,
) -> Result<B, E> {
    let to = to.min(source.len());
    let chunk_size = source.cursor_chunk_size().max(1);
    let mut buf = Vec::with_capacity(chunk_size.min(to.saturating_sub(from)));
    let mut acc = init;

    let mut start = from;
    while start < to {
        let end = chunk_end(start, chunk_size, to);
        source.read_into_at(start, end, &mut buf);
        for value in buf.drain(..) {
            acc = f(acc, value)?;
        }
        start = end;
    }

    Ok(acc)
}
