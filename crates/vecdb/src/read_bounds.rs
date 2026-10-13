use std::{cell::RefCell, collections::BTreeMap, marker::PhantomData, sync::Arc};

use crate::AnyExportableVec;

mod bounded_vec;
mod bounded_writer;

pub use bounded_vec::BoundedVec;
pub use bounded_writer::BoundedWriter;

thread_local! {
    static CURRENT: RefCell<Option<Arc<BTreeMap<&'static str, usize>>>> = const { RefCell::new(None) };
}

/// Per-index limits applied while serving one published read snapshot.
#[derive(Clone, Default)]
pub struct ReadBounds(Arc<BTreeMap<&'static str, usize>>);

/// Bounds this thread's reads until dropped, restoring the enclosing bounds. Not `Send`: the
/// bounds live in a thread-local.
#[must_use = "the bounds apply only while the guard lives"]
pub struct ReadScope {
    previous: Option<Arc<BTreeMap<&'static str, usize>>>,
    _thread: PhantomData<*const ()>,
}

impl ReadBounds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, index: &'static str, len: usize) {
        Arc::make_mut(&mut self.0).insert(index, len);
    }

    /// Bind an exported vector to explicit limits. An unspecified index is
    /// rejected rather than silently exposing the vector's full length.
    pub fn bind<'a>(&'a self, source: &'a dyn AnyExportableVec) -> Option<BoundedVec<'a>> {
        self.0
            .get(source.index_type_to_string())
            .map(|&limit| BoundedVec::new(source, self, limit))
    }

    pub fn scope<T>(&self, f: impl FnOnce() -> T) -> T {
        let _scope = self.enter();
        f()
    }

    /// Bounds this thread's reads until the returned guard drops.
    pub fn enter(&self) -> ReadScope {
        ReadScope {
            previous: CURRENT.with(|current| current.replace(Some(Arc::clone(&self.0)))),
            _thread: PhantomData,
        }
    }
}

impl Drop for ReadScope {
    fn drop(&mut self) {
        let previous = self.previous.take();
        CURRENT.with(|current| {
            current.replace(previous);
        });
    }
}

/// `len` capped by this thread's bound for `index`, if a [`ReadScope`] sets one.
pub fn visible_len(index: &str, len: usize) -> usize {
    CURRENT.with(|current| {
        current
            .borrow()
            .as_ref()
            .and_then(|bounds| bounds.get(index))
            .copied()
            .map_or(len, |bound| len.min(bound))
    })
}
