use bitview_collections::Windows;
use bitview_primitives::PartsPerMillionSigned64;
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, Sats, SatsSigned, Version};
use derive_more::{Deref, DerefMut};
use vecdb::ReadableCloneableVec;

use crate::{IndexSources, LazyRollingDeltasAmountFromHeight, LazySpotValuePerBlock};

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazySpotValuePerBlockWithDeltas {
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    inner: LazySpotValuePerBlock,
    delta: LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
}

impl LazySpotValuePerBlockWithDeltas {
    pub fn from_sats_source(
        name: &str,
        version: Version,
        source: &(impl ReadableCloneableVec<Height, Sats> + ?Sized),
        indexes: &IndexSources,
        window_starts: &Windows<&impl ReadableCloneableVec<Height, Height>>,
        spot_price: &impl ReadableCloneableVec<Height, Cents>,
    ) -> Self {
        let inner =
            LazySpotValuePerBlock::from_sats_source(name, version, source, indexes, spot_price);
        let delta = LazyRollingDeltasAmountFromHeight::new(
            &format!("{name}_delta"),
            version + Version::TWO,
            &inner.sats.height,
            window_starts,
            indexes,
        );
        Self { inner, delta }
    }
}
