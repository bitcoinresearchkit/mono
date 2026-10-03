use std::sync::Arc;

use vecdb::{ReadableCloneableVec, VecIndex, VecValue, Version};

#[derive(Clone)]
pub struct TerminalLen {
    get: Arc<dyn Fn() -> usize + Send + Sync>,
    version: Version,
}

impl TerminalLen {
    pub fn new<I, T>(source: &(impl ReadableCloneableVec<I, T> + ?Sized)) -> Self
    where
        I: VecIndex,
        T: VecValue,
    {
        let source = source.read_only_boxed_clone();
        let version = source.version();
        Self {
            get: Arc::new(move || source.visible_len()),
            version,
        }
    }

    #[inline(always)]
    pub fn get(&self) -> usize {
        (self.get)()
    }

    #[inline(always)]
    pub fn version(&self) -> Version {
        self.version
    }
}
