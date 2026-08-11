use brk_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{
    BinaryTransform, CachedBoxedVec, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, TypedVec,
};

use crate::{
    indexes,
    internal::{
        CentsUnsignedToDollars, DerivedResolutions, Identity, LazyIndexedVec, LazyPerBlock,
        ReadableResolutions, SatsToBitcoin, SatsToCents,
    },
};

/// Fully lazy point-in-time value backed by one sats source.
#[derive(Clone, Traversable)]
pub struct LazySpotValuePerBlock {
    pub btc: LazyPerBlock<Bitcoin, Sats>,
    pub sats: LazyPerBlock<Sats>,
    pub usd: LazyPerBlock<Dollars, Cents>,
    pub cents: LazyPerBlock<Cents>,
}

pub(crate) trait SpotValueSource {
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
    pub(crate) fn identity(name: &str, version: Version, source: &Self) -> Self {
        let sats = LazyPerBlock::from_lazy::<Identity<Sats>, Sats>(
            &format!("{name}_sats"),
            version,
            &source.sats,
        );
        let btc = LazyPerBlock::from_lazy::<SatsToBitcoin, Sats>(name, version, &source.sats);
        let cents = LazyPerBlock::from_lazy::<Identity<Cents>, Cents>(
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

    pub(crate) fn from_sats_source<V>(
        name: &str,
        version: Version,
        source: V,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self
    where
        V: TypedVec<I = Height, T = Sats> + ReadableVec<Height, Sats> + Clone + 'static,
    {
        let sats = LazyPerBlock::from_uncached_height_source::<Identity<Sats>, _>(
            &format!("{name}_sats"),
            version,
            source,
            indexes,
        );
        let btc = LazyPerBlock::from_lazy::<SatsToBitcoin, Sats>(name, version, &sats);
        let cents_source = LazyIndexedVec::new(
            &format!("{name}_cents_source"),
            version,
            sats.height.read_only_boxed_clone(),
            spot_price.clone(),
            |_, sats, spot| SatsToCents::apply(sats, spot),
        );
        let cents = LazyPerBlock::from_uncached_height_source::<Identity<Cents>, _>(
            &format!("{name}_cents"),
            version,
            cents_source,
            indexes,
        );
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

    pub(crate) fn from_boxed_sats_source(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, Sats>,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self {
        let sats = LazyPerBlock::from_uncached_boxed_height_source::<Identity<Sats>>(
            &format!("{name}_sats"),
            version,
            source,
            indexes,
        );
        let btc = LazyPerBlock::from_lazy::<SatsToBitcoin, Sats>(name, version, &sats);
        let cents_source = LazyIndexedVec::new(
            &format!("{name}_cents_source"),
            version,
            sats.height.read_only_boxed_clone(),
            spot_price.clone(),
            |_, sats, spot| SatsToCents::apply(sats, spot),
        );
        let cents = LazyPerBlock::from_uncached_height_source::<Identity<Cents>, _>(
            &format!("{name}_cents"),
            version,
            cents_source,
            indexes,
        );
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
