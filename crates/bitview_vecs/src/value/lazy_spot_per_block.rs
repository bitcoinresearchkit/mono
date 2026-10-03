use bitview_compute::WeightedCohortState;
use bitview_primitives::BoundedRatio;
use bitview_transforms::{CentsUnsignedToDollars, SatsToBitcoin, SatsToCents};
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{BinaryTransform, Ident, ReadableBoxedVec, ReadableCloneableVec};

use crate::{
    DerivedResolutions, IndexSources, LazyIndexedVec, LazyPerBlock, ReadableResolutions, Value,
};

/// Fully lazy point-in-time value backed by one sats source.
pub type LazySpotValuePerBlock = Value<
    LazyPerBlock<Sats>,
    LazyPerBlock<Cents>,
    LazyPerBlock<Bitcoin, Sats>,
    LazyPerBlock<Dollars, Cents>,
>;

pub trait SpotValueSource {
    type SatsResolutions: ReadableResolutions<Sats>;
    type CentsResolutions: ReadableResolutions<Cents>;
    type DollarsResolutions: ReadableResolutions<Dollars>;

    fn sats_height(&self) -> ReadableBoxedVec<Height, Sats>;
    fn cents_height(&self) -> ReadableBoxedVec<Height, Cents>;
    fn usd_height(&self) -> ReadableBoxedVec<Height, Dollars>;
    fn sats_resolutions(&self) -> &Self::SatsResolutions;
    fn cents_resolutions(&self) -> &Self::CentsResolutions;
    fn usd_resolutions(&self) -> &Self::DollarsResolutions;
}

impl SpotValueSource for LazySpotValuePerBlock {
    type SatsResolutions = DerivedResolutions<Sats>;
    type CentsResolutions = DerivedResolutions<Cents>;
    type DollarsResolutions = DerivedResolutions<Dollars, Cents>;

    fn sats_height(&self) -> ReadableBoxedVec<Height, Sats> {
        self.sats.height.read_only_boxed_clone()
    }

    fn cents_height(&self) -> ReadableBoxedVec<Height, Cents> {
        self.cents.height.read_only_boxed_clone()
    }

    fn usd_height(&self) -> ReadableBoxedVec<Height, Dollars> {
        self.usd.height.read_only_boxed_clone()
    }

    fn sats_resolutions(&self) -> &Self::SatsResolutions {
        &self.sats.resolutions
    }

    fn cents_resolutions(&self) -> &Self::CentsResolutions {
        &self.cents.resolutions
    }

    fn usd_resolutions(&self) -> &Self::DollarsResolutions {
        &self.usd.resolutions
    }
}

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
        let btc = LazyPerBlock::from_lazy::<SatsToBitcoin, Sats>(name, version, &source.sats);
        let cents = LazyPerBlock::from_lazy::<Ident, Cents>(
            &format!("{name}_cents"),
            version,
            &source.cents,
        );
        let usd = LazyPerBlock::from_lazy::<CentsUnsignedToDollars, Cents>(
            &format!("{name}_usd"),
            version,
            &source.cents,
        );

        Self {
            btc,
            sats,
            usd,
            cents,
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
        let sats = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_sats"),
            version,
            source,
            indexes,
        );
        Self::from_sats(name, version, sats, indexes, spot_price)
    }

    fn from_sats(
        name: &str,
        version: Version,
        sats: LazyPerBlock<Sats>,
        indexes: &IndexSources,
        spot_price: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Self {
        let cents_source = LazyIndexedVec::new(
            &format!("{name}_cents_source"),
            version,
            &sats.height,
            spot_price,
            |_, sats, spot| SatsToCents::apply(sats, spot),
        );
        let cents = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_cents"),
            version,
            &cents_source,
            indexes,
        );
        Self::from_sats_and_cents(name, version, sats, cents)
    }

    fn from_sats_and_cents(
        name: &str,
        version: Version,
        sats: LazyPerBlock<Sats>,
        cents: LazyPerBlock<Cents>,
    ) -> Self {
        let btc = LazyPerBlock::from_lazy::<SatsToBitcoin, Sats>(name, version, &sats);
        let usd = LazyPerBlock::from_lazy::<CentsUnsignedToDollars, Cents>(
            &format!("{name}_usd"),
            version,
            &cents,
        );

        Self {
            btc,
            sats,
            usd,
            cents,
        }
    }
}
