use crate::{AnyVec, READ_CHUNK_SIZE, ReadableVec, StoredVec, VecIndex, VecValue, Version};

use super::{
    ColumnId, ColumnarVec, LazyColumnVec, ReadOnlyColumnarVec, ReadableColumnarVec, rows_per_block,
    schema::validate_column,
};

/// Reads matrix rows from a flat page-blocked scalar vector.
pub(super) fn read_rows<I, T, R, C>(
    vec: &R,
    rows: usize,
    from: usize,
    to: usize,
    out: &mut Vec<C::Row<T>>,
) where
    I: VecIndex,
    T: VecValue,
    R: ReadableVec<I, T>,
    C: ColumnId,
{
    let from = from.min(rows);
    let to = to.min(rows);
    if from >= to {
        return;
    }

    let per_block = rows_per_block::<T>();
    let column_count = C::ALL.len();
    out.reserve(to - from);
    let capacity = per_block.min(to - from);
    let mut columns = C::from_fn(|_| Vec::with_capacity(capacity));
    let mut at = from;
    while at < to {
        let block = at / per_block;
        let block_start = block * per_block;
        let block_rows = per_block.min(rows - block_start);
        let local_from = at - block_start;
        let take = (to - at).min(block_rows - local_from);
        let flat_block_start = block * per_block * column_count;

        for &column in C::ALL {
            let values = column.get_mut(&mut columns);
            values.clear();
            let start = flat_block_start + column.index() * block_rows + local_from;
            vec.read_into_at(start, start + take, values);
        }
        out.extend((0..take).map(|index| C::from_fn(|column| column.get(&columns)[index].clone())));
        at += take;
    }
}

fn for_each_column_chunk<I, T, R, C, F>(
    vec: &R,
    rows: usize,
    columns: &[C],
    from: usize,
    to: usize,
    f: &mut F,
) where
    I: VecIndex,
    T: VecValue,
    R: ReadableVec<I, T>,
    C: ColumnId,
    F: FnMut(C, usize, &[T]),
{
    let from = from.min(rows);
    let to = to.min(rows);
    if from >= to {
        return;
    }

    let per_block = rows_per_block::<T>();
    let column_count = C::ALL.len();
    let mut values = Vec::with_capacity(per_block.min(to - from));
    let mut at = from;
    while at < to {
        let block = at / per_block;
        let block_start = block * per_block;
        let block_rows = per_block.min(rows - block_start);
        let local_from = at - block_start;
        let take = (to - at).min(block_rows - local_from);
        let flat_block_start = block * per_block * column_count;

        for &column in columns {
            values.clear();
            let start = flat_block_start + column.index() * block_rows + local_from;
            vec.read_into_at(start, start + take, &mut values);
            f(column, at, &values);
        }
        at += take;
    }
}

impl<V, C> ReadableColumnarVec<C> for ReadOnlyColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    type I = V::I;
    type T = V::T;

    fn for_each_column_chunk_at<F>(&self, columns: &[C], from: usize, to: usize, f: &mut F)
    where
        F: FnMut(C, usize, &[V::T]),
    {
        for &column in columns {
            validate_column(column);
        }
        let _guard = self.gate.read();
        for_each_column_chunk::<V::I, V::T, V::ReadOnly, C, F>(
            &self.vec,
            self.visible_rows.get(),
            columns,
            from,
            to,
            f,
        );
    }
}

impl<V, C> ReadableVec<V::I, C::Row<V::T>> for ColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    #[inline(always)]
    fn collect_one_at(&self, index: usize) -> Option<C::Row<V::T>> {
        if index >= self.stored_rows {
            return self.pushed.get(index - self.stored_rows).cloned();
        }

        let mut row = Vec::with_capacity(1);
        read_rows::<V::I, V::T, V, C>(&self.vec, self.flat_rows(), index, index + 1, &mut row);
        row.pop()
    }

    fn cursor_chunk_size(&self) -> usize {
        Self::ROWS_PER_BLOCK * READ_CHUNK_SIZE.div_ceil(Self::ROWS_PER_BLOCK)
    }

    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<C::Row<V::T>>) {
        let len = self.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return;
        }

        if from < self.stored_rows {
            read_rows::<V::I, V::T, V, C>(
                &self.vec,
                self.flat_rows(),
                from,
                to.min(self.stored_rows),
                out,
            );
        }
        if to > self.stored_rows {
            let start = from.max(self.stored_rows) - self.stored_rows;
            let end = to - self.stored_rows;
            out.extend_from_slice(&self.pushed[start..end]);
        }
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(C::Row<V::T>)) {
        fold_readable(self, from, to, (), |(), value| f(value));
    }

    fn fold_range_at<B, F: FnMut(B, C::Row<V::T>) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> B {
        fold_readable(self, from, to, init, f)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, C::Row<V::T>) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> std::result::Result<B, E> {
        try_fold_readable(self, from, to, init, f)
    }
}

impl<V, C> ReadableVec<V::I, C::Row<V::T>> for ReadOnlyColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    fn cursor_chunk_size(&self) -> usize {
        let per_block = rows_per_block::<V::T>();
        per_block * READ_CHUNK_SIZE.div_ceil(per_block)
    }

    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<C::Row<V::T>>) {
        let _guard = self.gate.read();
        read_rows::<V::I, V::T, V::ReadOnly, C>(&self.vec, self.visible_rows.get(), from, to, out);
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(C::Row<V::T>)) {
        fold_readable(self, from, to, (), |(), value| f(value));
    }

    fn fold_range_at<B, F: FnMut(B, C::Row<V::T>) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> B {
        fold_readable(self, from, to, init, f)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, C::Row<V::T>) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> std::result::Result<B, E> {
        try_fold_readable(self, from, to, init, f)
    }
}

impl<S, C> LazyColumnVec<S, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
{
    pub(super) fn new(name: &str, version: Version, source: S, column: C) -> Self {
        validate_column(column);
        Self {
            name: name.into(),
            base_version: version,
            source,
            column,
        }
    }
}

fn fold_column<S, C, B, F>(source: &S, column: C, from: usize, to: usize, init: B, mut f: F) -> B
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    F: FnMut(B, S::T) -> B,
{
    let mut acc = Some(init);
    source.for_each_column_chunk_at(&[column], from, to, &mut |_, _, values| {
        for value in values {
            acc = Some(f(
                acc.take().expect("column fold accumulator"),
                value.clone(),
            ));
        }
    });
    acc.expect("column fold accumulator")
}

fn try_fold_column<S, C, B, E, F>(
    source: &S,
    column: C,
    from: usize,
    to: usize,
    init: B,
    mut f: F,
) -> std::result::Result<B, E>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
    F: FnMut(B, S::T) -> std::result::Result<B, E>,
{
    let from = from.min(source.len());
    let to = to.min(source.len());
    let chunk_size = source.cursor_chunk_size().max(1);
    let mut acc = Some(init);
    let mut at = from;
    while at < to {
        let end = (at + chunk_size).min(to);
        let mut error = None;
        source.for_each_column_chunk_at(&[column], at, end, &mut |_, _, values| {
            if error.is_some() {
                return;
            }
            for value in values {
                let current = acc.take().expect("column fold accumulator");
                match f(current, value.clone()) {
                    Ok(next) => acc = Some(next),
                    Err(err) => {
                        error = Some(err);
                        break;
                    }
                }
            }
        });
        if let Some(error) = error {
            return Err(error);
        }
        at = end;
    }
    Ok(acc.expect("column fold accumulator"))
}

impl<S, C> ReadableVec<S::I, S::T> for LazyColumnVec<S, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
{
    fn cursor_chunk_size(&self) -> usize {
        self.source.cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<S::T>) {
        self.source
            .for_each_column_chunk_at(&[self.column], from, to, &mut |_, _, values| {
                out.extend_from_slice(values)
            });
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, f: &mut dyn FnMut(S::T)) {
        self.source
            .for_each_column_chunk_at(&[self.column], from, to, &mut |_, _, values| {
                for value in values {
                    f(value.clone());
                }
            });
    }

    fn fold_range_at<B, F: FnMut(B, S::T) -> B>(&self, from: usize, to: usize, init: B, f: F) -> B {
        fold_column(&self.source, self.column, from, to, init, f)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, S::T) -> std::result::Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> std::result::Result<B, E> {
        try_fold_column(&self.source, self.column, from, to, init, f)
    }
}

pub(super) fn fold_readable<I, T, R, B, F>(
    vec: &R,
    from: usize,
    to: usize,
    mut acc: B,
    mut f: F,
) -> B
where
    I: VecIndex,
    T: VecValue,
    R: ReadableVec<I, T>,
    F: FnMut(B, T) -> B,
{
    let from = from.min(vec.len());
    let to = to.min(vec.len());
    let chunk = vec.cursor_chunk_size().max(1);
    let mut buf = Vec::with_capacity(chunk.min(to.saturating_sub(from)));
    let mut at = from;
    while at < to {
        let end = (at + chunk).min(to);
        buf.clear();
        vec.read_into_at(at, end, &mut buf);
        for value in buf.drain(..) {
            acc = f(acc, value);
        }
        at = end;
    }
    acc
}

pub(super) fn try_fold_readable<I, T, R, B, E, F>(
    vec: &R,
    from: usize,
    to: usize,
    mut acc: B,
    mut f: F,
) -> std::result::Result<B, E>
where
    I: VecIndex,
    T: VecValue,
    R: ReadableVec<I, T>,
    F: FnMut(B, T) -> std::result::Result<B, E>,
{
    let from = from.min(vec.len());
    let to = to.min(vec.len());
    let chunk = vec.cursor_chunk_size().max(1);
    let mut buf = Vec::with_capacity(chunk.min(to.saturating_sub(from)));
    let mut at = from;
    while at < to {
        let end = (at + chunk).min(to);
        buf.clear();
        vec.read_into_at(at, end, &mut buf);
        for value in buf.drain(..) {
            acc = f(acc, value)?;
        }
        at = end;
    }
    Ok(acc)
}
