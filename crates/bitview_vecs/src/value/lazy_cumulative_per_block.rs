use bitview_transforms::Convert;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{Ident, ReadableCloneableVec};

use crate::{IndexSources, LazyPerBlock, Value};

pub type LazyCumulativeValuePerBlock = Value<
    LazyPerBlock<Sats>,
    LazyPerBlock<Cents>,
    LazyPerBlock<Bitcoin, Sats>,
    LazyPerBlock<Dollars, Cents>,
>;

impl LazyCumulativeValuePerBlock {
    pub(crate) fn from_sources(
        name: &str,
        version: Version,
        cumulative_sats: &(impl ReadableCloneableVec<Height, Sats> + ?Sized),
        cumulative_cents: &(impl ReadableCloneableVec<Height, Cents> + ?Sized),
        indexes: &IndexSources,
    ) -> Self {
        let sats = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_sats"),
            version,
            cumulative_sats,
            indexes,
        );
        let btc = LazyPerBlock::from_lazy::<Convert, Sats>(name, version, &sats);
        let cents = LazyPerBlock::from_height_source::<Ident>(
            &format!("{name}_cents"),
            version,
            cumulative_cents,
            indexes,
        );
        let usd =
            LazyPerBlock::from_lazy::<Convert, Cents>(&format!("{name}_usd"), version, &cents);
        Self {
            btc,
            sats,
            usd,
            cents,
        }
    }
}
