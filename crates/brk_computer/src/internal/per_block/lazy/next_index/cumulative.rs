use std::sync::Arc;

use brk_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::StoredU64;
use schemars::JsonSchema;
use serde::Serialize;
use vecdb::{
    AnyExportableVec, AnyVec, Cursor, Formattable, ReadableBoxedVec, ReadableVec, TypedVec,
    VecIndex, VecValue, Version, short_type_name,
};

use super::terminal_len::TerminalLen;

/// Inclusive cumulative count derived from the next value in one first-index
/// source. The final value uses live terminal-length metadata.
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
    pub fn new<TI, TT>(
        name: &str,
        version: Version,
        first_indexes: ReadableBoxedVec<I, S>,
        terminal: ReadableBoxedVec<TI, TT>,
    ) -> Self
    where
        TI: VecIndex,
        TT: VecValue,
    {
        Self {
            name: Arc::from(name),
            base_version: version,
            first_indexes,
            terminal_len: TerminalLen::new(terminal),
        }
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(StoredU64))
    where
        StoredU64: From<S>,
    {
        let len = self.len();
        let to = to.min(len);
        if from >= to {
            return;
        }

        self.first_indexes
            .for_each_range_dyn_at(from + 1, (to + 1).min(len), &mut |next| {
                each(StoredU64::from(next))
            });

        if to == len {
            each(StoredU64::from(S::from(self.terminal_len.get())));
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
        size_of::<StoredU64>()
    }

    fn value_type_to_string(&self) -> &'static str {
        short_type_name::<StoredU64>()
    }
}

impl<I, S> TypedVec for LazyCumulativeIndexVec<I, S>
where
    I: VecIndex,
    S: VecIndex,
{
    type I = I;
    type T = StoredU64;
}

impl<I, S> ReadableVec<I, StoredU64> for LazyCumulativeIndexVec<I, S>
where
    I: VecIndex,
    S: VecIndex,
    StoredU64: From<S>,
{
    fn cursor_chunk_size(&self) -> usize {
        self.first_indexes.cursor_chunk_size()
    }

    fn read_into_at(&self, from: usize, to: usize, buf: &mut Vec<StoredU64>) {
        buf.reserve(to.saturating_sub(from));
        self.for_each_value(from, to, |value| buf.push(value));
    }

    fn for_each_range_dyn_at(&self, from: usize, to: usize, each: &mut dyn FnMut(StoredU64)) {
        self.for_each_value(from, to, each);
    }

    fn fold_range_at<B, F: FnMut(B, StoredU64) -> B>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> B {
        let mut values = Vec::with_capacity(to.saturating_sub(from));
        self.read_into_at(from, to, &mut values);
        values.into_iter().fold(init, fold)
    }

    fn try_fold_range_at<B, E, F: FnMut(B, StoredU64) -> Result<B, E>>(
        &self,
        from: usize,
        to: usize,
        init: B,
        fold: F,
    ) -> Result<B, E> {
        let mut values = Vec::with_capacity(to.saturating_sub(from));
        self.read_into_at(from, to, &mut values);
        values.into_iter().try_fold(init, fold)
    }

    fn collect_one_at(&self, index: usize) -> Option<StoredU64> {
        if index >= self.len() {
            return None;
        }
        let next = if index + 1 < self.len() {
            self.first_indexes.collect_one_at(index + 1)?
        } else {
            S::from(self.terminal_len.get())
        };
        Some(StoredU64::from(next))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<StoredU64>) {
        let len = self.len();
        let terminal = S::from(self.terminal_len.get());
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
            out.push(StoredU64::from(next));
        }
    }
}

impl<I, S> Traversable for LazyCumulativeIndexVec<I, S>
where
    I: VecIndex,
    S: VecIndex,
    StoredU64: From<S> + Formattable + Serialize + JsonSchema,
{
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf::<I, StoredU64, _>(self)
    }
}
