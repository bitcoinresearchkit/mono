use brk_types::{Cents, Height, PartsPerMillionSigned64, Sats, Version};
use vecdb::{
    BinaryTransform, CachedBoxedVec, ReadableCloneableVec, ReadableVec, TypedVec, VecValue,
};

use crate::internal::{LazyIndexedVec, LazyWindowVec, SatsToCents};

use super::metrics::AllSupplyCache;

/// Shared handles to the pinned all-chain inputs.
///
/// Cloning these handles does not duplicate either cached array.
#[derive(Clone)]
pub(crate) struct AllChainCache {
    supply: CachedBoxedVec<Height, Sats>,
    price: CachedBoxedVec<Height, Cents>,
}

#[derive(Clone, Debug)]
struct WithSupply<S> {
    source: S,
    supply: Sats,
}

#[derive(Clone, Debug, Default)]
struct MarketAndRealizedCap {
    market: Cents,
    realized: Cents,
}

impl AllChainCache {
    pub(crate) fn new(supply: &AllSupplyCache, price: &CachedBoxedVec<Height, Cents>) -> Self {
        Self {
            supply: supply.cached_boxed_clone(),
            price: price.clone(),
        }
    }

    /// Lazily combines one ordinary source with the pinned all-supply cache.
    pub(crate) fn with_supply<S, T>(
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
    pub(crate) fn with_market_cap<S, T>(
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

    /// Computes market-cap growth minus realized-cap growth from one realized
    /// cap source and cached window starts.
    pub(crate) fn market_minus_realized_cap_growth(
        &self,
        name: &str,
        version: Version,
        realized_cap: &(impl ReadableCloneableVec<Height, Cents> + 'static),
        window_starts: CachedBoxedVec<Height, Height>,
    ) -> impl TypedVec<I = Height, T = PartsPerMillionSigned64>
    + ReadableVec<Height, PartsPerMillionSigned64>
    + Clone
    + 'static {
        let caps = self.with_market_cap(
            &format!("{name}_caps"),
            Version::ZERO,
            realized_cap,
            |_, realized, market| MarketAndRealizedCap { market, realized },
        );

        LazyWindowVec::new(
            name,
            version,
            caps.read_only_boxed_clone(),
            window_starts,
            false,
            |current, previous, _| {
                let growth = |current: Cents, previous: Cents| {
                    if previous == Cents::ZERO {
                        0.0
                    } else {
                        (f64::from(current) - f64::from(previous)) / f64::from(previous)
                    }
                };
                PartsPerMillionSigned64::from(
                    growth(current.market, previous.market)
                        - growth(current.realized, previous.realized),
                )
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use brk_types::PartsPerMillionSigned64;
    use vecdb::{
        AnyStoredVec, CachedVec, Database, EagerVec, ImportableVec, PcoVec, ReadOnlyClone,
        WritableVec,
    };

    use super::*;

    #[test]
    fn derives_from_one_source_and_shared_chain_caches() {
        let suffix = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "brk-all-chain-cache-{}-{suffix}",
            std::process::id()
        ));
        let db = Database::open(&path).unwrap();

        let mut supply: EagerVec<PcoVec<Height, Sats>> =
            EagerVec::forced_import(&db, "supply", Version::ONE).unwrap();
        let mut price: EagerVec<PcoVec<Height, Cents>> =
            EagerVec::forced_import(&db, "price", Version::ONE).unwrap();
        let mut realized: EagerVec<PcoVec<Height, Cents>> =
            EagerVec::forced_import(&db, "realized", Version::ONE).unwrap();
        let mut starts: EagerVec<PcoVec<Height, Height>> =
            EagerVec::forced_import(&db, "starts", Version::ONE).unwrap();

        for value in [100_000_000, 100_000_000, 200_000_000] {
            supply.push(Sats::new(value));
        }
        for value in [100, 200, 200] {
            price.push(Cents::new(value));
        }
        for value in [50, 100, 100] {
            realized.push(Cents::new(value));
        }
        for value in [0, 0, 1] {
            starts.push(Height::new(value));
        }
        supply.write().unwrap();
        price.write().unwrap();
        realized.write().unwrap();
        starts.write().unwrap();

        let supply_cache = AllSupplyCache::new(supply.read_only_clone());
        let price_cache = CachedVec::wrap(price);
        let cache = AllChainCache::new(&supply_cache, &price_cache.read_only_cached_boxed_clone());

        let cached_supply =
            cache.with_supply("cached_supply", Version::ONE, &realized, |_, _, supply| {
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

        let market_cap = cache.with_market_cap(
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

        let starts_cache = CachedVec::wrap(starts);
        let growth = cache.market_minus_realized_cap_growth(
            "growth",
            Version::ONE,
            &realized,
            starts_cache.read_only_cached_boxed_clone(),
        );
        assert_eq!(
            growth.collect_range(Height::ZERO, Height::new(3)),
            [
                PartsPerMillionSigned64::ZERO,
                PartsPerMillionSigned64::ZERO,
                PartsPerMillionSigned64::ONE,
            ],
        );

        drop(growth);
        drop(starts_cache);
        drop(market_cap);
        drop(cached_supply);
        drop(cache);
        drop(price_cache);
        drop(supply_cache);
        drop(realized);
        drop(supply);
        drop(db);
        std::fs::remove_dir_all(path).unwrap();
    }
}
