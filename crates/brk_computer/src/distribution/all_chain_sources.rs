use brk_types::{Cents, Height, Sats, Version};
use vecdb::{
    BinaryTransform, CachedBoxedVec, ReadableCloneableVec, ReadableVec, TypedVec, VecValue,
};

use crate::internal::{LazyIndexedVec, SatsToCents};

/// Shared handles to the pinned all-chain inputs.
///
/// Cloning these handles does not duplicate either cached array.
#[derive(Clone)]
pub struct AllChainSources {
    supply: CachedBoxedVec<Height, Sats>,
    price: CachedBoxedVec<Height, Cents>,
}

#[derive(Clone, Debug)]
struct WithSupply<S> {
    source: S,
    supply: Sats,
}

impl AllChainSources {
    pub fn new(
        supply: &CachedBoxedVec<Height, Sats>,
        price: &CachedBoxedVec<Height, Cents>,
    ) -> Self {
        Self {
            supply: supply.clone(),
            price: price.clone(),
        }
    }

    /// Lazily combines one ordinary source with the pinned all-supply cache.
    pub fn with_supply<S, T>(
        &self,
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, S> + 'static),
        compute: impl Fn(Height, S, Sats) -> T + Send + Sync + 'static,
    ) -> LazyIndexedVec<Height, S, Sats, T>
    where
        S: VecValue,
        T: VecValue,
    {
        LazyIndexedVec::new(
            name,
            version,
            source.read_only_boxed_clone(),
            self.supply.clone(),
            compute,
        )
    }

    /// Lazily combines one ordinary source with market cap derived from the
    /// pinned all-supply and spot-price caches.
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
        let with_supply = self.with_supply(
            &format!("{name}_with_supply"),
            Version::ZERO,
            source,
            |_, source, supply| WithSupply { source, supply },
        );

        LazyIndexedVec::new(
            name,
            version,
            with_supply.read_only_boxed_clone(),
            self.price.clone(),
            move |height, with_supply, price| {
                compute(
                    height,
                    with_supply.source,
                    SatsToCents::apply(with_supply.supply, price),
                )
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use vecdb::{
        AnyStoredVec, CachedReadableVec, CachedVec, Database, EagerVec, ImportableVec, PcoVec,
        ReadOnlyClone, WritableVec,
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
        let mut price: EagerVec<PcoVec<Height, Cents>> =
            EagerVec::forced_import(&db, "price", Version::ONE).unwrap();
        let mut realized: EagerVec<PcoVec<Height, Cents>> =
            EagerVec::forced_import(&db, "realized", Version::ONE).unwrap();

        for value in [100_000_000, 100_000_000, 200_000_000] {
            supply.push(Sats::new(value));
        }
        for value in [100, 200, 200] {
            price.push(Cents::new(value));
        }
        for value in [50, 100, 100] {
            realized.push(Cents::new(value));
        }
        supply.write().unwrap();
        price.write().unwrap();
        realized.write().unwrap();

        let supply_cache = CachedVec::wrap(supply.read_only_clone()).cached_boxed_clone();
        let price_cache = CachedVec::wrap(price);
        let sources =
            AllChainSources::new(&supply_cache, &price_cache.read_only_cached_boxed_clone());

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
        drop(price_cache);
        drop(supply_cache);
        drop(realized);
        drop(supply);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
