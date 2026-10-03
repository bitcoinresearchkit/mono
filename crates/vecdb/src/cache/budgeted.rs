use std::sync::{Arc, OnceLock};

use crate::{Error, Result, VecValue};

use super::{Cache, CacheBudget, CachePolicy};

static BUDGET: OnceLock<CacheBudget> = OnceLock::new();

/// Retain source-selected ranges within the immutable process-wide budget.
#[derive(Clone, Copy, Debug, Default)]
pub struct Budgeted;

impl Budgeted {
    /// Initialize once at startup, before importing any budgeted source.
    pub fn init_global(bytes: usize) -> Result<&'static CacheBudget> {
        BUDGET.set(CacheBudget::new(bytes)).map_err(|_| {
            Error::InvalidArgument("the global cache budget is already initialized")
        })?;
        Self::global()
    }

    fn global() -> Result<&'static CacheBudget> {
        BUDGET.get().ok_or(Error::InvalidArgument(
            "initialize the global cache budget before importing budgeted vectors",
        ))
    }
}

impl CachePolicy for Budgeted {
    type State<T: VecValue> = Option<Arc<Cache<T>>>;

    fn create<T: VecValue>() -> Result<Self::State<T>> {
        let budget = Self::global()?;
        Ok((budget.limit() > 0).then(|| Cache::new(budget)))
    }

    #[inline(always)]
    fn cache<T: VecValue>(state: &Self::State<T>) -> Option<&Cache<T>> {
        state.as_deref()
    }
}
