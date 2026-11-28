use std::{iter, sync::Arc};

use bitview_traversable::{Traversable, TreeNode, make_leaf};
use brk_types::Height;
use vecdb::{
    AnyExportableVec, AnyVec, ReadOnlyClone, ReadableBoxedVec, ReadableCloneableVec, ReadableVec,
    TypedVec, VecIndex, VecValue, Version,
};

use crate::{HeightLookup, HeightMap};

/// The height of the block holding each row's index: `source` read once, each value looked up in
/// the resident block boundaries; null where `source` is.
#[derive(Clone)]
pub struct HeightOfVec<I: VecIndex, K: VecIndex + VecValue> {
    name: Arc<str>,
    source: ReadableBoxedVec<I, Option<K>>,
    heights: HeightLookup<K>,
}

impl<I: VecIndex, K: VecIndex + VecValue> HeightOfVec<I, K> {
    pub fn new(
        name: &str,
        source: &(impl ReadableCloneableVec<I, Option<K>> + ?Sized),
        heights: &HeightLookup<K>,
    ) -> Self {
        Self {
            name: Arc::from(name),
            source: source.read_only_boxed_clone(),
            heights: heights.clone(),
        }
    }

    fn height(map: &HeightMap<K>, key: Option<K>) -> Option<Height> {
        key.map(|key| map.get(key).expect("block boundaries cover every indexed key"))
    }

    fn try_for_each_value<E>(
        &self,
        from: usize,
        to: usize,
        mut each: impl FnMut(Option<Height>) -> Result<(), E>,
    ) -> Result<(), E> {
        let keys = self.source.collect_range_at(from, to);
        let map = self.heights.load();
        keys.into_iter().try_for_each(|key| each(Self::height(&map, key)))
    }

    fn for_each_value(&self, from: usize, to: usize, mut each: impl FnMut(Option<Height>)) {
        let Ok(()) = self.try_for_each_value(from, to, |value| {
            each(value);
            Ok::<_, !>(())
        });
    }
}

impl<I: VecIndex, K: VecIndex + VecValue> AnyVec for HeightOfVec<I, K> {
    fn version(&self) -> Version {
        self.source.version()
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn len(&self) -> usize {
        self.source.len()
    }

    fn visible_len(&self) -> usize {
        self.source.visible_len()
    }

    fn is_mutable(&self) -> bool {
        self.source.is_mutable()
    }

    vecdb::any_vec_type_methods!(I, Option<Height>);
}

impl<I: VecIndex, K: VecIndex + VecValue> TypedVec for HeightOfVec<I, K> {
    type I = I;
    type T = Option<Height>;
}

impl<I: VecIndex, K: VecIndex + VecValue> ReadableVec<I, Option<Height>> for HeightOfVec<I, K> {
    vecdb::visitor_read_methods!(Option<Height>, for_each_value, try_for_each_value);

    fn collect_one_at(&self, index: usize) -> Option<Option<Height>> {
        let key = self.source.collect_one_at(index)?;
        Some(Self::height(&self.heights.load(), key))
    }

    fn read_sorted_into_at(&self, indices: &[usize], out: &mut Vec<Option<Height>>) {
        let keys = self.source.read_sorted_at(indices);
        let map = self.heights.load();
        out.extend(keys.into_iter().map(|key| Self::height(&map, key)));
    }
}

impl<I: VecIndex, K: VecIndex + VecValue> ReadOnlyClone for HeightOfVec<I, K> {
    type ReadOnly = Self;

    fn read_only_clone(&self) -> Self {
        self.clone()
    }
}

impl<I: VecIndex, K: VecIndex + VecValue> Traversable for HeightOfVec<I, K> {
    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        iter::once(self as &dyn AnyExportableVec)
    }

    fn to_tree_node(&self) -> TreeNode {
        make_leaf(self)
    }
}
