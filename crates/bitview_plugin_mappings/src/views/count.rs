use std::{convert::Infallible, iter, sync::Arc};

use bitview_primitives::StoredU64;
use bitview_traversable::{Traversable, TreeNode, make_leaf};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, CheckedSub, Cursor, Formattable, ReadableBoxedVec,
    ReadableCloneableVec, ReadableVec, SparseRead, TypedVec, VecIndex, VecValue, Version,
    short_type_name,
};

use super::terminal_len::TerminalLen;

/// Per-item count derived by subtracting adjacent values in one first-index
/// source. Range reads borrow source chunks and carry their shared boundary.
#[derive(Clone)]
pub struct LazyIndexCountVec<I, S>
where
    I: VecIndex,
    S: VecIndex,
{
    name: Arc<str>,
    base_version: Version,
    first_indexes: ReadableBoxedVec<I, S>,
    terminal_len: TerminalLen,
}

impl<I, S> LazyIndexCountVec<I, S>
where
    I: VecIndex,
    S: VecIndex + CheckedSub + Default,
    StoredU64: From<S>,
{
    pub(crate) fn new<TI, TT>(
        name: &str,
        version: Version,
        first_indexes: &(impl ReadableCloneableVec<I, S> + ?Sized),
        terminal: &(impl ReadableCloneableVec<TI, TT> + ?Sized),
    ) -> Self
    where
        TI: VecIndex,
        TT: VecValue,
    {
        Self {
            name: Arc::from(name),
            base_version: version,
            first_indexes: first_indexes.read_only_boxed_clone(),
            terminal_len: TerminalLen::new(terminal),
        }
    }

    #[inline(always)]
    fn count(current: S, next: S) -> StoredU64 {
        StoredU64::from(next.checked_sub(current).unwrap_or_default())
    }

    fn for_each_input(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(Option<StoredU64>, &[S]),
    ) {
        let len = self.len();
        let to = to.min(len);
        if from >= to {
            return;
        }
        let terminal = (to == len).then(|| S::from(self.terminal_len.get()));
        let mut previous = None;
        self.first_indexes
            .for_each_chunk_at(from, (to + 1).min(len), &mut |_, boundaries| {
                if let Some(&first) = boundaries.first() {
                    each(
                        previous.map(|previous| Self::count(previous, first)),
                        boundaries,
                    );
                    previous = boundaries.last().copied();
                }
            });
        if let (Some(previous), Some(terminal)) = (previous, terminal) {
            each(Some(Self::count(previous, terminal)), &[]);
        }
    }

    fn try_fold_values<B, E>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: impl FnMut(B, StoredU64) -> Result<B, E>,
    ) -> Result<B, E> {
        let mut accumulator = Some(Ok(init));
        self.for_each_input(from, to, |first, boundaries| {
            accumulator = Some(accumulator.take().unwrap().and_then(|accumulator| {
                let accumulator = if let Some(first) = first {
                    fold(accumulator, first)?
                } else {
                    accumulator
                };
                boundaries
                    .windows(2)
                    .try_fold(accumulator, |accumulator, pair| {
                        fold(accumulator, Self::count(pair[0], pair[1]))
                    })
            }));
        });
        accumulator.unwrap()
    }
}

impl<I, S> AnyVec for LazyIndexCountVec<I, S>
where
    I: VecIndex,
    S: VecIndex,
{
    fn version(&self) -> Version {
        self.base_version + self.first_indexes.version() + self.terminal_len.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.first_indexes.len()
    }

    fn index_type_to_string(&self) -> &'static str {
        I::to_string()
    }

    fn region_names(&self) -> Vec<String> {
        Vec::new()
    }

    fn value_type_to_size_of(&self) -> usize {
        size_of::<StoredU64>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<StoredU64>()
    }
}

impl<I, S> TypedVec for LazyIndexCountVec<I, S>
where
    I: VecIndex,
    S: VecIndex,
{
    type I = I;
    type T = StoredU64;
}

impl<I, S> ReadableVec<I, StoredU64> for LazyIndexCountVec<I, S>
where
    I: VecIndex,
    S: VecIndex + CheckedSub + Default,
    StoredU64: From<S>,
{
    fn cursor_chunk_size(&self) -> usize {
        self.first_indexes.cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<StoredU64>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_input(from, to, |first, boundaries| {
            buf.extend(first);
            buf.extend(
                boundaries
                    .windows(2)
                    .map(|pair| Self::count(pair[0], pair[1])),
            );
        });
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(StoredU64)) {
        self.fold_range_at(from, to, (), |(), value| each(value));
    }

    fn fold_range_at<B, F: FnMut(B, StoredU64) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        self.try_fold_values(from, to, init, |accumulator, value| {
            Ok::<_, Infallible>(fold(accumulator, value))
        })
        .unwrap()
    }

    fn try_fold_range_at<B, E, F: FnMut(B, StoredU64) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        self.try_fold_values(from, to, init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<StoredU64> {
        self.fold_range_at(index, index.saturating_add(1), None, |_, value| Some(value))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<StoredU64>) {
        let len = self.len();
        let terminal = S::from(self.terminal_len.get());
        let indices = &indices[..indices.partition_point(|&i| i < len)];
        if indices.len() > 1
            && indices.windows(2).all(|pair| pair[1] == pair[0] + 1)
            && !self.first_indexes.is_mutable()
        {
            self.read_into_at(indices[0], indices[indices.len() - 1] + 1, out);
            return;
        }
        if let (Some(&first), Some(&last)) = (indices.first(), indices.last())
            && last - first >= indices.len()
            && !self.first_indexes.is_mutable()
            && let Some(values) = SparseRead::try_new(&*self.first_indexes, indices, |i| {
                (i + 1 < len).then_some(i + 1)
            })
        {
            out.extend((0..indices.len()).map(|slot| {
                Self::count(
                    values.current(slot),
                    values.previous(slot).unwrap_or(terminal),
                )
            }));
            return;
        }
        let mut first_indexes = Cursor::new(&*self.first_indexes);
        let mut previous = None;

        out.reserve(indices.len());
        for &index in indices {
            if index >= len {
                break;
            }
            if let Some((previous_index, value)) = previous
                && previous_index == index
            {
                out.push(value);
                continue;
            }

            let Some(current) = first_indexes.get(index) else {
                continue;
            };
            let next = if index + 1 < len {
                let Some(next) = first_indexes.get(index + 1) else {
                    continue;
                };
                next
            } else {
                terminal
            };
            let value = Self::count(current, next);
            previous = Some((index, value));
            out.push(value);
        }
    }
}

impl<I, S> Traversable for LazyIndexCountVec<I, S>
where
    I: VecIndex,
    S: VecIndex + CheckedSub + Default,
    StoredU64: From<S> + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, StoredU64, _>(self)
    }
}
