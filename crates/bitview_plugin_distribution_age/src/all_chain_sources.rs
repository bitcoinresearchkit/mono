use bitview_vecs::LazyIndexedVec;
use brk_types::{Cents, Height, Sats, Version};
use vecdb::{ReadableBoxedVec, ReadableCloneableVec, VecValue};

/// Shared views of all-chain inputs. Retention belongs to their stored sources.
#[derive(Clone)]
pub struct AllChainSources {
    supply: ReadableBoxedVec<Height, Sats>,
    market_cap: ReadableBoxedVec<Height, Cents>,
}

impl AllChainSources {
    pub fn new(
        supply: &impl ReadableCloneableVec<Height, Sats>,
        market_cap: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Self {
        Self {
            supply: supply.read_only_boxed_clone(),
            market_cap: market_cap.read_only_boxed_clone(),
        }
    }

    /// Derive from shared inputs. The caller owns the ordinary source's cache.
    pub fn with_supply<S, T>(
        &self,
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        compute: impl Fn(Height, S, Sats) -> T + Send + Sync + 'static,
    ) -> LazyIndexedVec<Height, S, Sats, T>
    where
        S: VecValue,
        T: VecValue,
    {
        LazyIndexedVec::new(name, version, source, &self.supply, compute)
    }

    /// Derive from shared inputs without caching another full-height result.
    pub fn with_market_cap<S, T>(
        &self,
        name: &str,
        version: Version,
        source: &impl ReadableCloneableVec<Height, S>,
        compute: impl Fn(Height, S, Cents) -> T + Send + Sync + 'static,
    ) -> LazyIndexedVec<Height, S, Cents, T>
    where
        S: VecValue,
        T: VecValue,
    {
        LazyIndexedVec::new(name, version, source, &self.market_cap, compute)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_cache::init_cache;

    use std::{
        env, fs, process,
        time::{SystemTime, UNIX_EPOCH},
    };

    use vecdb::{
        AnyStoredVec, Budgeted, Database, EagerVec, ImportableVec, PcoVec, ReadableCloneableVec,
        ReadableVec, WritableVec,
    };

    use super::*;

    #[test]
    fn derives_from_shared_chain_sources() {
        init_cache();
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path =
            env::temp_dir().join(format!("brk-all-chain-sources-{}-{suffix}", process::id()));
        let db = Database::open(&path).unwrap();

        let mut supply: EagerVec<PcoVec<Height, Sats, Budgeted>> =
            EagerVec::forced_import(&db, "supply", Version::ONE).unwrap();
        let mut market_cap: EagerVec<PcoVec<Height, Cents, Budgeted>> =
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

        let supply_cache = supply.read_only_boxed_clone();
        let market_cap_cache = market_cap.read_only_boxed_clone();
        let sources =
            AllChainSources::new(&supply_cache, &market_cap_cache.read_only_boxed_clone());

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
        fs::remove_dir_all(path).unwrap();
    }
}
