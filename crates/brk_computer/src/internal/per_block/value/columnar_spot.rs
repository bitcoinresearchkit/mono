use brk_traversable::Traversable;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{
    BinaryTransform, CachedBoxedVec, ColumnId, PcoVec, ReadOnlyColumnarVec, ReadableCloneableVec,
};

use crate::{
    indexes,
    internal::{
        CACHE_BUDGET, CentsUnsignedToDollars, Identity, LazyColumnPerBlock, LazyIndexedVec,
        LazyPerBlock, SatsToBitcoin, SatsToCents,
    },
};

#[derive(Clone, Traversable)]
pub struct LazyColumnSpotValuePerBlock<C>
where
    C: ColumnId,
{
    pub btc: LazyPerBlock<Bitcoin, Sats>,
    pub sats: LazyColumnPerBlock<Sats, C>,
    pub usd: LazyPerBlock<Dollars, Cents>,
    pub cents: LazyPerBlock<Cents>,
}

impl<C> LazyColumnSpotValuePerBlock<C>
where
    C: ColumnId,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, Sats>, C>,
        column: C,
        indexes: &indexes::Vecs,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self {
        let sats =
            LazyColumnPerBlock::new(&format!("{name}_sats"), version, source, column, indexes);
        let btc = LazyPerBlock::from_resolutions::<SatsToBitcoin>(
            name,
            version,
            sats.height.read_only_boxed_clone(),
            &sats.resolutions,
        );
        let cents_source = LazyIndexedVec::new(
            &format!("{name}_cents_source"),
            version,
            sats.height.read_only_boxed_clone(),
            spot_price.clone(),
            |_, sats, spot| SatsToCents::apply(sats, spot),
        );
        let cents_source = CACHE_BUDGET.wrap(cents_source);
        let cents = LazyPerBlock::from_height_source::<Identity<Cents>>(
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
