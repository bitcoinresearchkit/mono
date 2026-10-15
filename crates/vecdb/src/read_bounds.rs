use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    marker::PhantomData,
    sync::Arc,
};

use crate::AnyExportableVec;

mod bounded_vec;
mod bounded_writer;

pub use bounded_vec::BoundedVec;
pub use bounded_writer::BoundedWriter;

type Bounds = Arc<BTreeMap<&'static str, usize>>;

thread_local! {
    /// This thread's live scopes, newest last, each with its number of guards: the newest applies.
    static SCOPES: RefCell<Vec<(u64, Bounds, usize)>> = const { RefCell::new(Vec::new()) };
    static NEXT_SCOPE: Cell<u64> = const { Cell::new(0) };
}

/// Per-index limits applied while serving one published read snapshot.
#[derive(Clone, Default)]
pub struct ReadBounds(Bounds);

/// Bounds this thread's reads until dropped; the newest live scope applies, whatever order scopes
/// drop in. Not `Send`: the bounds live in a thread-local.
#[must_use = "the bounds apply only while the guard lives"]
pub struct ReadScope {
    id: u64,
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

    /// Bounds this thread's reads until the returned guard drops. Entering the bounds that
    /// already apply joins their scope (no registration): a writer's scope per value inside
    /// one around the whole serialization costs a check.
    pub fn enter(&self) -> ReadScope {
        let id = SCOPES.with(|scopes| {
            let mut scopes = scopes.borrow_mut();
            if let Some((id, bounds, guards)) = scopes.last_mut()
                && Arc::ptr_eq(bounds, &self.0)
            {
                *guards += 1;
                return *id;
            }
            let id = NEXT_SCOPE.replace(NEXT_SCOPE.get() + 1);
            scopes.push((id, Arc::clone(&self.0), 1));
            id
        });
        ReadScope {
            id,
            _thread: PhantomData,
        }
    }
}

impl Drop for ReadScope {
    fn drop(&mut self) {
        SCOPES.with(|scopes| {
            let mut scopes = scopes.borrow_mut();
            if let Some(at) = scopes.iter().rposition(|(id, ..)| *id == self.id) {
                scopes[at].2 -= 1;
                if scopes[at].2 == 0 {
                    scopes.remove(at);
                }
            }
        });
    }
}

/// `len` capped by this thread's bound for `index`, if a [`ReadScope`] sets one.
pub fn visible_len(index: &str, len: usize) -> usize {
    SCOPES.with(|scopes| {
        scopes
            .borrow()
            .last()
            .and_then(|(_, bounds, _)| bounds.get(index))
            .copied()
            .map_or(len, |bound| len.min(bound))
    })
}
