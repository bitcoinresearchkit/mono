use brk_types::{Cents, Height, Sats, Version};
use vecdb::{
    BinaryTransform, CachedBoxedVec, CachedReadableVec, CachedVec, LazyVec, ReadableCloneableVec,
    ReadableVec, TypedVec,
};

use crate::{
    indexes,
    internal::{Identity, LazyIndexedVec, LazyPerBlock, SatsToCents},
};

use super::LazySpotValuePerBlock;

#[derive(Clone)]
pub struct PinnedSpotValuePerBlock {
    pub series: LazySpotValuePerBlock,
    pub sats: CachedBoxedVec<Height, Sats>,
    pub cents: CachedBoxedVec<Height, Cents>,
}

impl PinnedSpotValuePerBlock {
    pub fn from_sats_source<V>(
        name: &str,
        version: Version,
        source: V,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self
    where
        V: TypedVec<I = Height, T = Sats> + ReadableVec<Height, Sats> + Clone + 'static,
    {
        let sats_name = format!("{name}_sats");
        let sats_height = LazyVec::transformed::<Identity<Sats>>(
            &sats_name,
            Version::ZERO,
            source.read_only_boxed_clone(),
        );
        let sats_height = CachedVec::wrap(sats_height);
        let sats_cache = sats_height.cached_boxed_clone();
        let sats = LazyPerBlock::from_height_source::<Identity<Sats>>(
            &sats_name,
            version,
            sats_height,
            indexes,
        );

        let cents_name = format!("{name}_cents");
        let cents_source = LazyIndexedVec::new(
            &format!("{cents_name}_source"),
            version,
            sats.height.read_only_boxed_clone(),
            spot_price.clone(),
            |_, sats, spot| SatsToCents::apply(sats, spot),
        );
        let cents_height = CachedVec::wrap(cents_source);
        let cents_cache = cents_height.cached_boxed_clone();
        let cents = LazyPerBlock::from_height_source::<Identity<Cents>>(
            &cents_name,
            version,
            cents_height,
            indexes,
        );

        Self {
            series: LazySpotValuePerBlock::from_sats_and_cents(name, version, sats, cents),
            sats: sats_cache,
            cents: cents_cache,
        }
    }
}
