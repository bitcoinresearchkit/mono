use bitview_vecs::LazyIndexedVec;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec, VecValue};

/// Shared views of all-chain inputs. Retention belongs to their stored sources.
#[derive(Clone)]
pub struct AllChainSources {
    supply: ReadableBoxedVec<Height, Sats>,
    market_cap: ReadableBoxedVec<Height, Cents>,
    realized_cap: ReadableBoxedVec<Height, Cents>,
}

impl AllChainSources {
    pub fn new(
        supply: &impl ReadableCloneableVec<Height, Sats>,
        market_cap: &impl ReadableCloneableVec<Height, Cents>,
        realized_cap: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Self {
        Self {
            supply: supply.read_only_boxed_clone(),
            market_cap: market_cap.read_only_boxed_clone(),
            realized_cap: realized_cap.read_only_boxed_clone(),
        }
    }

    pub fn realized_cap(&self) -> &ReadableBoxedVec<Height, Cents> {
        &self.realized_cap
    }

    /// Derive from shared inputs. The caller owns the ordinary source's cache.
    pub fn with_supply<S, T, F>(
        &self,
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        compute: F,
    ) -> LazyIndexedVec<Height, S, Sats, T, F>
    where
        S: VecValue,
        T: VecValue,
        F: Fn(Height, S, Sats) -> T + Clone + Send + Sync + 'static,
    {
        LazyIndexedVec::new(name, version, source, &self.supply, compute)
    }

    /// Derive from shared inputs without caching another full-height result.
    pub fn with_market_cap<S, T, F>(
        &self,
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        compute: F,
    ) -> LazyIndexedVec<Height, S, Cents, T, F>
    where
        S: VecValue,
        T: VecValue,
        F: Fn(Height, S, Cents) -> T + Clone + Send + Sync + 'static,
    {
        LazyIndexedVec::new(name, version, source, &self.market_cap, compute)
    }
}
