use brk_types::{Cents, Height, Sats, Version};
use vecdb::{CachedBoxedVec, ReadableCloneableVec, ReadableVec, TypedVec, VecValue};

use bitview_compute::{CACHE_BUDGET, LazyIndexedVec};

/// Shared handles to the pinned all-chain inputs.
///
/// Cloning these handles does not duplicate either cached array.
#[derive(Clone)]
pub struct AllChainSources {
    supply: CachedBoxedVec<Height, Sats>,
    market_cap: CachedBoxedVec<Height, Cents>,
}

impl AllChainSources {
    pub fn new(
        supply: &CachedBoxedVec<Height, Sats>,
        market_cap: &CachedBoxedVec<Height, Cents>,
    ) -> Self {
        Self {
            supply: supply.clone(),
            market_cap: market_cap.clone(),
        }
    }

    /// Combines one ordinary source with pinned all supply, caching the result
    /// when it becomes hot.
    pub fn with_supply<S, T>(
        &self,
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + 'static),
        compute: impl Fn(Height, S, Sats) -> T + Send + Sync + 'static,
    ) -> impl TypedVec<I = Height, T = T> + ReadableVec<Height, T> + Clone + 'static
    where
        S: VecValue,
        T: VecValue,
    {
        CACHE_BUDGET.wrap(LazyIndexedVec::new(
            name,
            version,
            source.read_only_boxed_clone(),
            self.supply.clone(),
            compute,
        ))
    }

    /// Combines one ordinary source with market cap, caching only the final
    /// result when it becomes hot.
    pub fn with_market_cap<S, T>(
        &self,
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + 'static),
        compute: impl Fn(Height, S, Cents) -> T + Send + Sync + 'static,
    ) -> impl TypedVec<I = Height, T = T> + ReadableVec<Height, T> + Clone + 'static
    where
        S: VecValue,
        T: VecValue,
    {
        CACHE_BUDGET.wrap(LazyIndexedVec::new(
            name,
            version,
            source.read_only_boxed_clone(),
            self.market_cap.clone(),
            compute,
        ))
    }
}

#[cfg(test)]
mod tests {
    use vecdb::{
        AnyStoredVec, CachedReadableVec, CachedVec, Database, EagerVec, ImportableVec, PcoVec,
        ReadOnlyClone, ReadableVec, WritableVec,
    };

    use super::*;

    #[test]
    fn derives_from_shared_chain_sources() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "brk-all-chain-sources-{}-{suffix}",
            std::process::id()
        ));
        let db = Database::open(&path).unwrap();

        let mut supply: EagerVec<PcoVec<Height, Sats>> =
            EagerVec::forced_import(&db, "supply", Version::ONE).unwrap();
        let mut market_cap: EagerVec<PcoVec<Height, Cents>> =
            EagerVec::forced_import(&db, "market_cap", Version::ONE).unwrap();
        let mut realized: EagerVec<PcoVec<Height, Cents>> =
            EagerVec::forced_import(&db, "realized", Version::ONE).unwrap();

        for value in [100_000_000, 100_000_000, 200_000_000] {
            supply.push(Sats::new(value));
        }
        for value in [100, 200, 400] {
            market_cap.push(Cents::new(value));
        }
        for value in [50, 100, 100] {
            realized.push(Cents::new(value));
        }
        supply.write().unwrap();
        market_cap.write().unwrap();
        realized.write().unwrap();

        let supply_cache = CachedVec::wrap(supply.read_only_clone()).cached_boxed_clone();
        let market_cap_cache = CachedVec::wrap(market_cap);
        let sources = AllChainSources::new(
            &supply_cache,
            &market_cap_cache.read_only_cached_boxed_clone(),
        );

        let cached_supply =
            sources.with_supply("cached_supply", Version::ONE, &realized, |_, _, supply| {
                supply
            });
        assert_eq!(
            cached_supply.collect_range(Height::ZERO, Height::new(3)),
            [
                Sats::new(100_000_000),
                Sats::new(100_000_000),
                Sats::new(200_000_000)
            ],
        );

        let market_cap = sources.with_market_cap(
            "market_cap",
            Version::ONE,
            &realized,
            |_, realized, market| (realized, market),
        );
        assert_eq!(
            market_cap.collect_range(Height::ZERO, Height::new(3)),
            [
                (Cents::new(50), Cents::new(100)),
                (Cents::new(100), Cents::new(200)),
                (Cents::new(100), Cents::new(400)),
            ],
        );

        drop(market_cap);
        drop(cached_supply);
        drop(market_cap_cache);
        drop(supply_cache);
        drop(realized);
        drop(supply);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
