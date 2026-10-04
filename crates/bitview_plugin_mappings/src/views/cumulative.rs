use std::{convert::Infallible, iter, sync::Arc};

use bitview_primitives::Count;
use bitview_traversable::{Traversable, TreeNode, make_leaf};
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, Cursor, Formattable, ReadableBoxedVec, ReadableCloneableVec,
    ReadableVec, TypedVec, VecIndex, VecValue, Version, short_type_name,
};

use super::terminal_len::TerminalLen;

/// Inclusive cumulative count derived from the next value in one first-index
/// source. The final value uses the terminal length visible to the read.
#[derive(Clone)]
pub struct LazyCumulativeIndexVec<I, S>
where
    I: VecIndex,
    S: VecIndex,
{
    name: Arc<str>,
    base_version: Version,
    first_indexes: ReadableBoxedVec<I, S>,
    terminal_len: TerminalLen,
}

impl<I, S> LazyCumulativeIndexVec<I, S>
where
    I: VecIndex,
    S: VecIndex,
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

    fn for_each_input(&self, from: usize, to: usize, mut each: impl FnMut(&[S])) {
        let len = self.len();
        let to = to.min(len);
        if from >= to {
            return;
        }

        let terminal = (to == len).then(|| S::from(self.terminal_len.get()));
        self.first_indexes
            .for_each_chunk_at(from + 1, (to + 1).min(len), &mut |_, next| each(next));

        if let Some(terminal) = terminal {
            each(&[terminal]);
        }
    }
}

impl<I, S> AnyVec for LazyCumulativeIndexVec<I, S>
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
        size_of::<Count>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<Count>()
    }
}

impl<I, S> TypedVec for LazyCumulativeIndexVec<I, S>
where
    I: VecIndex,
    S: VecIndex,
{
    type I = I;
    type T = Count;
}

impl<I, S> ReadableVec<I, Count> for LazyCumulativeIndexVec<I, S>
where
    I: VecIndex,
    S: VecIndex,
    Count: From<S>,
{
    fn cursor_chunk_size(&self) -> usize {
        self.first_indexes.cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<Count>) {
        buf.reserve(to.min(self.len()).saturating_sub(from));
        self.for_each_input(from, to, |values| {
            buf.extend(values.iter().copied().map(Count::from));
        });
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(Count)) {
        self.fold_range_at(from, to, (), |(), value| each(value));
    }

    fn fold_range_at<B, F: FnMut(B, Count) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> B {
        self.try_fold_range_at(from, to, init, |accumulator, value| {
            Ok::<_, Infallible>(fold(accumulator, value))
        })
        .unwrap()
    }

    fn try_fold_range_at<B, E, F: FnMut(B, Count) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        mut fold: F,
    ) -> Result<B, E> {
        let mut accumulator = Some(Ok(init));
        self.for_each_input(from, to, |values| {
            accumulator = Some(accumulator.take().unwrap().and_then(|accumulator| {
                values
                    .iter()
                    .copied()
                    .map(Count::from)
                    .try_fold(accumulator, &mut fold)
            }));
        });
        accumulator.unwrap()
    }

    fn collect_one_at(&self, index: usize) -> Option<Count> {
        if index >= self.len() {
            return None;
        }
        let next = if index + 1 < self.len() {
            self.first_indexes.collect_one_at(index + 1)?
        } else {
            S::from(self.terminal_len.get())
        };
        Some(Count::from(next))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<Count>) {
        let len = self.len();
        let terminal = S::from(self.terminal_len.get());
        let indices = &indices[..indices.partition_point(|&i| i < len)];
        if let (Some(&first), Some(&last)) = (indices.first(), indices.last())
            && last - first >= indices.len()
            && !self.first_indexes.is_mutable()
        {
            let split = indices.partition_point(|&i| i + 1 < len);
            let next: Vec<_> = indices[..split].iter().map(|&i| i + 1).collect();
            out.extend(
                self.first_indexes
                    .read_sorted_at(&next)
                    .into_iter()
                    .map(Count::from),
            );
            out.extend(indices[split..].iter().map(|_| Count::from(terminal)));
            return;
        }
        let mut first_indexes = Cursor::new(&*self.first_indexes);

        out.reserve(indices.len());
        for &index in indices {
            if index >= len {
                break;
            }
            let next = if index + 1 < len {
                let Some(next) = first_indexes.get(index + 1) else {
                    continue;
                };
                next
            } else {
                terminal
            };
            out.push(Count::from(next));
        }
    }
}

impl<I, S> Traversable for LazyCumulativeIndexVec<I, S>
where
    I: VecIndex,
    S: VecIndex,
    Count: From<S> + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}
