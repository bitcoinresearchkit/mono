use std::ops::{Deref, DerefMut};

use vecdb::{
    AnyExportableVec, LazyVec, ReadOnlyClone, ReadableCloneableVec, Ro, Rw, StorageMode, StoredVec,
    UnaryTransform, Version,
};

use crate::{Traversable, TreeNode};

/// A series stored in a compact type and published as a wider one: the stored vec is hidden and
/// a view converting each value carries its id. Writers and readers of the stored vec go through
/// `Deref`.
pub struct Compact<V: StoredVec + 'static, T, M: StorageMode = Rw> {
    stored: M::Stored<V>,
    view: LazyVec<V::I, T, V::I, V::T>,
}

impl<V, T> Compact<V, T>
where
    V: StoredVec + ReadableCloneableVec<V::I, V::T> + 'static,
    V::T: Into<T>,
    T: vecdb::VecValue,
{
    pub fn new(stored: V) -> Self {
        let view = LazyVec::transformed::<Widen>(
            stored.name(),
            Version::ZERO,
            stored.read_only_boxed_clone(),
        );
        Self { stored, view }
    }
}

impl<V: StoredVec + 'static, T, M: StorageMode> Compact<V, T, M> {
    /// The published view, for plugins that show the same series beside their own.
    pub fn view(&self) -> &LazyVec<V::I, T, V::I, V::T> {
        &self.view
    }
}

impl<V: StoredVec + 'static, T, M: StorageMode> Deref for Compact<V, T, M> {
    type Target = M::Stored<V>;

    fn deref(&self) -> &Self::Target {
        &self.stored
    }
}

impl<V: StoredVec + 'static, T, M: StorageMode> DerefMut for Compact<V, T, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stored
    }
}

impl<V, T, M> Traversable for Compact<V, T, M>
where
    V: StoredVec + 'static,
    M: StorageMode,
    M::Stored<V>: Traversable,
    LazyVec<V::I, T, V::I, V::T>: Traversable,
{
    fn to_tree_node(&self) -> TreeNode {
        self.view.to_tree_node()
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.stored
            .iter_any_exportable()
            .chain(self.view.iter_any_exportable())
    }

    fn iter_any_visible(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.view.iter_any_visible()
    }
}

impl<V: StoredVec + 'static, T: Clone> ReadOnlyClone for Compact<V, T> {
    type ReadOnly = Compact<V, T, Ro>;

    fn read_only_clone(&self) -> Self::ReadOnly {
        Compact {
            stored: self.stored.read_only_clone(),
            view: self.view.clone(),
        }
    }
}

/// A lossless widening through `Into`, compiled into the view's read loop.
struct Widen;

impl<A: Into<B>, B> UnaryTransform<A, B> for Widen {
    #[inline(always)]
    fn apply(value: A) -> B {
        value.into()
    }
}
