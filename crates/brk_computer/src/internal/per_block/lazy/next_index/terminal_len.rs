use std::sync::Arc;

use vecdb::{ReadableBoxedVec, VecIndex, VecValue, Version};

#[derive(Clone)]
pub struct TerminalLen {
    get: Arc<dyn Fn() -> usize + Send + Sync>,
    version: Version,
}

impl TerminalLen {
    pub fn new<I, T>(source: ReadableBoxedVec<I, T>) -> Self
    where
        I: VecIndex,
        T: VecValue,
    {
        let version = source.version();
        Self {
            get: Arc::new(move || source.len()),
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
