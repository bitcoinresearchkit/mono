use std::fmt::Debug;

use crate::{Error, ReadableVec, Result, VecIndex, VecValue, Version};

use super::{LazyColumnSumVec, LazyColumnVec};

/// Typed description of a fixed column set and its logical row representation.
pub trait ColumnId: Copy + Debug + Eq + Ord + Send + Sync + 'static {
    type Row<T>: VecValue
    where
        T: VecValue;

    /// Bump this whenever the column meaning or physical ordering changes.
    const VERSION: Version;

    /// Every valid column in physical storage order.
    const ALL: &'static [Self];

    fn index(self) -> usize;

    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T;

    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T;

    fn from_fn<T, F>(f: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T;

    fn map<T, U, F>(row: Self::Row<T>, f: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U;
}

/// Read-only access to a source that preserves typed column boundaries.
pub trait ReadableColumnarVec<C>: ReadableVec<Self::I, C::Row<Self::T>> + Clone
where
    C: ColumnId,
{
    type I: VecIndex;
    type T: VecValue;

    /// Visits selected columns in the requested order within row-aligned chunks.
    ///
    /// `row_start` is the absolute index of the first value in `values`.
    fn for_each_column_chunk_at<F>(&self, columns: &[C], from: usize, to: usize, f: &mut F)
    where
        F: FnMut(C, usize, &[Self::T]);

    fn column(&self, name: &str, version: Version, column: C) -> LazyColumnVec<Self, C>
    where
        Self: Sized,
    {
        LazyColumnVec::new(name, version, self.clone(), column)
    }

    fn sum_columns(
        &self,
        name: &str,
        version: Version,
        columns: impl IntoIterator<Item = C>,
    ) -> LazyColumnSumVec<Self, C>
    where
        Self: Sized,
    {
        LazyColumnSumVec::new(name, version, self.clone(), columns)
    }
}

pub(super) fn validate_schema<C: ColumnId>() -> Result<()> {
    if C::ALL.is_empty() {
        return Err(Error::InvalidArgument(
            "ColumnarVec requires at least one column",
        ));
    }
    for (index, &column) in C::ALL.iter().enumerate() {
        if column.index() != index {
            return Err(Error::InvalidArgument(
                "ColumnId::ALL must contain every column in physical index order",
            ));
        }
    }
    Ok(())
}

pub(super) fn validate_column<C: ColumnId>(column: C) {
    let index = column.index();
    assert_eq!(
        C::ALL.get(index),
        Some(&column),
        "invalid column ID at physical index {index}",
    );
}
