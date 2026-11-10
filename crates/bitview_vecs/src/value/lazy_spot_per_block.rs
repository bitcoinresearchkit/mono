use bitview_compute::WeightedCohortState;
use bitview_primitives::BoundedRatio;
use bitview_traversable::NamedIdentity;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{AnyVec, Convert, Ident, ReadableCloneableVec};

use crate::{IndexSources, LazyIndexedVec, LazyPerBlock, SpotValue};

/// Point-in-time value of one sats source; USD is the sats at the spot price.
pub type LazySpotValuePerBlock =
    SpotValue<LazyPerBlock<Sats>, LazyPerBlock<Bitcoin, Sats>, LazyPerBlock<Dollars>>;

impl LazySpotValuePerBlock {
    fn from_parts(
        name: &str,
        btc: LazyPerBlock<Bitcoin, Sats>,
        sats: LazyPerBlock<Sats>,
        usd: LazyPerBlock<Dollars>,
    ) -> Self {
        let usd_name = usd.height.name();
        let cap_name = if usd_name.ends_with("_usd") {
            name.split('_')
                .map(|word| if word == "supply" { "market_cap" } else { word })
                .collect::<Vec<_>>()
                .join("_")
        } else {
            usd_name.to_owned()
        };
        let market_cap = NamedIdentity::new(cap_name, usd.clone());
        Self {
            btc,
            sats,
            usd,
            market_cap,
        }
    }

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
        let usd =
            LazyPerBlock::from_lazy::<Ident, Dollars>(&format!("{name}_usd"), version, &source.usd);

        Self::from_parts(name, btc, sats, usd)
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
            &format!("{name}_usd"),
            version,
            source,
            indexes,
            spot_price,
        )
    }

    /// Like [`Self::from_sats_source`], with the USD series under its own name (e.g. market cap
    /// for the circulating supply).
    pub fn from_sats_source_named<V>(
        name: &str,
        usd_name: &str,
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
        let usd =
            LazyPerBlock::from_height_source::<Ident>(usd_name, version, &usd_source, indexes);

        Self::from_parts(name, btc, sats, usd)
    }
}
