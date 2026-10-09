use bitview_compute::WeightedCohortState;
use bitview_primitives::BoundedRatio;
use bitview_transforms::{Convert, SatsToCents};
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{BinaryTransform, Ident, ReadableCloneableVec, UnaryTransform};

use crate::{IndexSources, LazyIndexedVec, LazyPerBlock, SpotValue};

/// Fully lazy point-in-time value backed by one sats source; USD is sats at the spot price.
pub type LazySpotValuePerBlock =
    SpotValue<LazyPerBlock<Sats>, LazyPerBlock<Bitcoin, Sats>, LazyPerBlock<Dollars>>;

impl LazySpotValuePerBlock {
    /// A lazy weighted stock from shared age-cohort inputs. Retains the historical
    /// independent flooring of the weighted and complementary sides.
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

        Self { btc, sats, usd }
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
            |_, sats, spot| {
                <Convert as UnaryTransform<Cents, Dollars>>::apply(SatsToCents::apply(sats, spot))
            },
        );
        let usd = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_usd"),
            version,
            &usd_source,
            indexes,
        );

        Self { btc, sats, usd }
    }
}
