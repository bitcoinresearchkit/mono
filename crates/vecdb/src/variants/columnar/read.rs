use crate::{AnyVec, ReadableVec, StoredVec, VecIndex, VecValue};

use super::{
    ColumnId, ColumnarVec, LazyColumnVec, ReadOnlyColumnarVec, ReadableColumnarVec,
    schema::validate_column,
};

fn read_rows<I, T, R, C>(
    columns: &[R],
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

    debug_assert_eq!(columns.len(), C::ALL.len());
    out.reserve(to - from);
    let chunk_size = columns[0].cursor_chunk_size().max(1);
    let mut values = C::from_fn(|_| Vec::with_capacity(chunk_size.min(to - from)));
    let mut at = from;
    while at < to {
        let end = (at + chunk_size).min(to);
        let take = end - at;
        for &column in C::ALL {
            let column_values = column.get_mut(&mut values);
            column_values.clear();
            columns[column.index()].read_into_at(at, end, column_values);
        }
        assert!(
            C::ALL
                .iter()
                .all(|&column| column.get(&values).len() == take),
            "column read returned incomplete rows",
        );
        out.extend((0..take).map(|index| C::from_fn(|column| column.get(&values)[index].clone())));
        at = end;
    }
}

fn for_each_column<I, T, R, C, F>(
    sources: &[R],
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

    let mut values = Vec::with_capacity(to - from);
    for &column in columns {
        values.clear();
        sources[column.index()].read_into_at(from, to, &mut values);
        assert_eq!(
            values.len(),
            to - from,
            "column read returned incomplete rows"
        );
        f(column, from, &values);
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
        for_each_column::<V::I, V::T, V::ReadOnly, C, F>(
            &self.columns,
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
        let stored_rows = self.stored_rows();
        if index >= stored_rows {
            return self.pushed.get(index - stored_rows).cloned();
        }

        let mut row = Vec::with_capacity(1);
        read_rows::<V::I, V::T, V, C>(&self.columns, stored_rows, index, index + 1, &mut row);
        row.pop()
    }

    fn cursor_chunk_size(&self) -> usize {
        self.first().cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<C::Row<V::T>>) {
        let len = self.len();
        let from = from.min(len);
        let to = to.min(len);
        if from >= to {
            return;
        }

        let stored_rows = self.stored_rows();
        if from < stored_rows {
            read_rows::<V::I, V::T, V, C>(
                &self.columns,
                stored_rows,
                from,
                to.min(stored_rows),
                out,
            );
        }
        if to > stored_rows {
            let start = from.max(stored_rows) - stored_rows;
            let end = to - stored_rows;
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

    fn try_fold_range_at<B, E, F: FnMut(B, C::Row<V::T>) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        try_fold_readable(self, from, to, init, f)
    }
}

impl<V, C> ReadableVec<V::I, C::Row<V::T>> for ReadOnlyColumnarVec<V, C>
where
    V: StoredVec,
    C: ColumnId,
{
    fn cursor_chunk_size(&self) -> usize {
        self.columns[0].cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, out: &mut Vec<C::Row<V::T>>) {
        let _guard = self.gate.read();
        read_rows::<V::I, V::T, V::ReadOnly, C>(
            &self.columns,
            self.visible_rows.get(),
            from,
            to,
            out,
        );
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

    fn try_fold_range_at<B, E, F: FnMut(B, C::Row<V::T>) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        try_fold_readable(self, from, to, init, f)
    }
}

impl<S, C> LazyColumnVec<S, C>
where
    C: ColumnId,
    S: ReadableColumnarVec<C>,
{
    fn fold_column<B, F>(&self, from: usize, to: usize, init: B, mut f: F) -> B
    where
        F: FnMut(B, S::T) -> B,
    {
        let mut acc = Some(init);
        self.source
            .for_each_column_chunk_at(&[self.column], from, to, &mut |_, _, values| {
                for value in values {
                    acc = Some(f(
                        acc.take().expect("column fold accumulator"),
                        value.clone(),
                    ));
                }
            });
        acc.expect("column fold accumulator")
    }

    fn try_fold_column<B, E, F>(&self, from: usize, to: usize, init: B, mut f: F) -> Result<B, E>
    where
        F: FnMut(B, S::T) -> Result<B, E>,
    {
        let from = from.min(self.source.len());
        let to = to.min(self.source.len());
        let chunk_size = self.source.cursor_chunk_size().max(1);
        let mut acc = Some(init);
        let mut at = from;
        while at < to {
            let end = (at + chunk_size).min(to);
            let mut error = None;
            self.source
                .for_each_column_chunk_at(&[self.column], at, end, &mut |_, _, values| {
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
        self.fold_column(from, to, init, f)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, S::T) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        self.try_fold_column(from, to, init, f)
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
) -> Result<B, E>
where
    I: VecIndex,
    T: VecValue,
    R: ReadableVec<I, T>,
    F: FnMut(B, T) -> Result<B, E>,
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
