//! Lazy value wrapper for point-in-time value sources.

use bitview_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use derive_more::{Deref, DerefMut};
use vecdb::UnaryTransform;

use crate::{Identity, LazyValue, LazyValueDerivedResolutions, SatsToBitcoin, SpotValueSource};

/// Lazy value wrapper with height + all derived last transforms.
#[derive(Clone, Deref, DerefMut, Traversable)]
#[traversable(merge)]
pub struct LazyValuePerBlock {
    #[traversable(flatten)]
    pub height: LazyValue<Height>,
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub resolutions: Box<LazyValueDerivedResolutions>,
}

impl LazyValuePerBlock {
    pub fn spot_identity(name: &str, source: &impl SpotValueSource, version: Version) -> Self {
        Self::from_spot_block_source::<
            Identity<Sats>,
            SatsToBitcoin,
            Identity<Cents>,
            Identity<Dollars>,
        >(name, source, version)
    }

    pub fn from_spot_block_source<
        SatsTransform,
        BitcoinTransform,
        CentsTransform,
        DollarsTransform,
    >(
        name: &str,
        source: &impl SpotValueSource,
        version: Version,
    ) -> Self
    where
        SatsTransform: UnaryTransform<Sats, Sats>,
        BitcoinTransform: UnaryTransform<Sats, Bitcoin>,
        CentsTransform: UnaryTransform<Cents, Cents>,
        DollarsTransform: UnaryTransform<Dollars, Dollars>,
    {
        let height = LazyValue::from_spot_block_source::<
            SatsTransform,
            BitcoinTransform,
            CentsTransform,
            DollarsTransform,
        >(name, source, version);

        let resolutions = LazyValueDerivedResolutions::from_spot_block_source::<
            SatsTransform,
            BitcoinTransform,
            CentsTransform,
            DollarsTransform,
        >(name, source, version);

        Self {
            height,
            resolutions: Box::new(resolutions),
        }
    }
}
