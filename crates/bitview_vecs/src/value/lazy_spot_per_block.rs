use bitview_compute::WeightedCohortState;
use bitview_primitives::BoundedRatio;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{Convert, Ident, ReadableCloneableVec};

use crate::{IndexSources, LazyIndexedVec, LazyPerBlock, SpotValue};

/// Point-in-time value of one sats source; USD is the sats at the spot price.
pub type LazySpotValuePerBlock =
    SpotValue<LazyPerBlock<Sats>, LazyPerBlock<Bitcoin, Sats>, LazyPerBlock<Dollars>>;

impl LazySpotValuePerBlock {
    /// A lazy weighted stock from shared age-cohort inputs, split as the stored aggregates are
    /// ([`WeightedCohortState::split_supply`]).
    pub fn from_weighted_supply<const COMPLEMENT: bool>(
        name: &str,
        version: Version,
        supply: &impl ReadableCloneableVec<Height, Sats>,
        weight: &impl ReadableCloneableVec<Height, BoundedRatio>,
        indexes: &IndexSources,
        spot: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Self {
        let source = LazyIndexedVec::new(
            &format!("{name}_sats"),
            version,
            supply,
            weight,
            |_, supply, weight| {
                let (weighted, complement) = WeightedCohortState::split_supply(supply, weight);
                if COMPLEMENT { complement } else { weighted }
            },
        );
        Self::from_sats_source(name, version, &source, indexes, spot)
    }

    pub fn identity(name: &str, version: Version, source: &Self) -> Self {
        let sats =
            LazyPerBlock::from_lazy::<Ident, Sats>(&format!("{name}_sats"), version, &source.sats);
        let btc = LazyPerBlock::from_lazy::<Convert, Sats>(name, version, &source.sats);
        let dollars =
            |name: &str| LazyPerBlock::from_lazy::<Ident, Dollars>(name, version, &source.usd);

        Self {
            btc,
            sats,
            usd: dollars(&format!("{name}_usd")),
            market_cap: dollars(&market_cap_name(name)),
        }
    }

    pub fn from_sats_source<V>(
        name: &str,
        version: Version,
        source: &V,
        indexes: &IndexSources,
        spot_price: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Self
    where
        V: ReadableCloneableVec<Height, Sats> + ?Sized,
    {
        Self::from_sats_source_named(
            name,
            &market_cap_name(name),
            version,
            source,
            indexes,
            spot_price,
        )
    }

    /// Like [`Self::from_sats_source`], with the market cap under its own name (`market_cap` for
    /// the circulating supply).
    pub fn from_sats_source_named<V>(
        name: &str,
        market_cap: &str,
        version: Version,
        source: &V,
        indexes: &IndexSources,
        spot_price: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Self
    where
        V: ReadableCloneableVec<Height, Sats> + ?Sized,
    {
        let sats = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_sats"),
            version,
            source,
            indexes,
        );
        let btc = LazyPerBlock::from_lazy::<Convert, Sats>(name, version, &sats);
        // Exact cents first, so dollars match the stored cents-backed values elsewhere.
        let usd_source = LazyIndexedVec::new(
            &format!("{name}_usd_source"),
            version,
            &sats.height,
            spot_price,
            |_, sats: Sats, spot| Dollars::from(sats.value_at(spot)),
        );
        let dollars = |name: &str| {
            LazyPerBlock::from_height_source::<Ident>(name, version, &usd_source, indexes)
        };

        Self {
            btc,
            sats,
            usd: dollars(&format!("{name}_usd")),
            market_cap: dollars(market_cap),
        }
    }
}

/// A supply stock's market-cap id: its id with `supply` read as `market_cap`
/// (`age_1y_supply_in_loss` is `age_1y_market_cap_in_loss`).
fn market_cap_name(name: &str) -> String {
    let words = name.split('_').collect::<Vec<_>>();
    assert_eq!(
        words.iter().filter(|&&word| word == "supply").count(),
        1,
        "a spot value names one supply: {name}"
    );
    words
        .into_iter()
        .map(|word| if word == "supply" { "market_cap" } else { word })
        .collect::<Vec<_>>()
        .join("_")
}
