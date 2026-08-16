use std::{cell::RefCell, collections::BTreeMap, sync::Arc};

thread_local! {
    static CURRENT: RefCell<Option<Arc<BTreeMap<&'static str, usize>>>> = const { RefCell::new(None) };
}

/// Per-index limits applied while serving one published read snapshot.
#[derive(Clone, Default)]
pub struct ReadBounds(Arc<BTreeMap<&'static str, usize>>);

struct ScopeGuard(Option<Arc<BTreeMap<&'static str, usize>>>);

impl ReadBounds {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, index: &'static str, len: usize) {
        Arc::make_mut(&mut self.0).insert(index, len);
    }

    pub fn scope<T>(&self, f: impl FnOnce() -> T) -> T {
        let previous = CURRENT.with(|current| current.replace(Some(Arc::clone(&self.0))));
        let _guard = ScopeGuard(previous);
        f()
    }
}

impl Drop for ScopeGuard {
    fn drop(&mut self) {
        let previous = self.0.take();
        CURRENT.with(|current| {
            current.replace(previous);
        });
    }
}

pub(crate) fn visible_len(index: &str, len: usize) -> usize {
    CURRENT.with(|current| {
        current
            .borrow()
            .as_ref()
            .and_then(|bounds| bounds.get(index))
            .copied()
            .map_or(len, |bound| len.min(bound))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scope_applies_and_restores_bounds() {
        let mut outer = ReadBounds::new();
        outer.set("height", 10);
        let mut inner = ReadBounds::new();
        inner.set("height", 4);

        assert_eq!(visible_len("height", 20), 20);
        outer.scope(|| {
            assert_eq!(visible_len("height", 20), 10);
            assert_eq!(visible_len("tx_index", 20), 20);
            inner.scope(|| assert_eq!(visible_len("height", 20), 4));
            assert_eq!(visible_len("height", 20), 10);
        });
        assert_eq!(visible_len("height", 20), 20);
    }
}
