use brk_traversable::Traversable;
use brk_types::{Cents, Height, PartsPerMillionSigned64, Sats, SatsSigned, Version};
use derive_more::{Deref, DerefMut};
use vecdb::{CachedBoxedVec, ReadableBoxedVec};

use crate::{
    indexes,
    internal::{CachedWindowStartVec, LazyRollingDeltasAmountFromHeight, Windows},
};

use super::LazySpotValuePerBlock;

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazySpotValuePerBlockWithDeltas {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub inner: LazySpotValuePerBlock,
    pub delta: LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
}

impl LazySpotValuePerBlockWithDeltas {
    pub(crate) fn from_boxed_sats_source(
        name: &str,
        version: Version,
        source: ReadableBoxedVec<Height, Sats>,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self {
        let inner = LazySpotValuePerBlock::from_boxed_sats_source(
            name, version, source, indexes, spot_price,
        );
        let delta = LazyRollingDeltasAmountFromHeight::new(
            &format!("{name}_delta"),
            version + Version::TWO,
            &inner.sats.height,
            cached_starts,
            indexes,
        );
        Self { inner, delta }
    }
}
