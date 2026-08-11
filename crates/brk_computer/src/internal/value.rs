use brk_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{LazyVec, UnaryTransform, VecIndex};

use crate::internal::SpotValueSource;

/// Fully lazy value type at height level.
///
/// All fields are lazy transforms from existing sources - no storage.
#[derive(Clone, Traversable)]
pub struct LazyValue<I: VecIndex> {
    pub btc: LazyVec<I, Bitcoin, I, Sats>,
    pub sats: LazyVec<I, Sats, I, Sats>,
    pub usd: LazyVec<I, Dollars, I, Dollars>,
    pub cents: LazyVec<I, Cents, I, Cents>,
}

impl LazyValue<Height> {
    pub(crate) fn from_spot_block_source<
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
        let sats = LazyVec::transformed::<SatsTransform>(
            &format!("{name}_sats"),
            version,
            source.sats_height(),
        );

        let btc = LazyVec::transformed::<BitcoinTransform>(name, version, source.sats_height());

        let cents = LazyVec::transformed::<CentsTransform>(
            &format!("{name}_cents"),
            version,
            source.cents_height(),
        );

        let usd = LazyVec::transformed::<DollarsTransform>(
            &format!("{name}_usd"),
            version,
            source.usd_height(),
        );

        Self {
            btc,
            sats,
            usd,
            cents,
        }
    }
}
