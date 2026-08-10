use brk_traversable::Traversable;
use brk_types::{
    Bitcoin, Cents, Dollars, Height, PartsPerMillionSigned64, Sats, SatsSigned, Version,
};
use derive_more::{Deref, DerefMut};
use vecdb::{
    BinaryTransform, CachedBoxedVec, ColumnId, PcoVec, ReadOnlyColumnarVec, ReadableCloneableVec,
};

use crate::{
    indexes,
    internal::{
        CachedWindowStartVec, CentsUnsignedToDollars, Identity, LazyColumnPerBlock, LazyIndexedVec,
        LazyPerBlock, LazyRollingAvgsAmountFromHeight, LazyRollingDeltasAmountFromHeight,
        LazyRollingSumsAmountFromHeight, LazyValueBlock, SatsToBitcoin, SatsToCents, Windows,
    },
};

#[derive(Clone, Traversable)]
pub struct LazyColumnValuePerBlock<C>
where
    C: ColumnId,
{
    pub btc: LazyPerBlock<Bitcoin, Sats>,
    pub sats: LazyColumnPerBlock<Sats, C>,
    pub usd: LazyPerBlock<Dollars, Cents>,
    pub cents: LazyColumnPerBlock<Cents, C>,
}

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
        let cents = LazyPerBlock::from_height_source::<Identity<Cents>, _>(
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

#[derive(Clone, Deref, DerefMut, Traversable)]
pub struct LazyColumnSpotValuePerBlockWithDeltas<C>
where
    C: ColumnId,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub inner: LazyColumnSpotValuePerBlock<C>,
    pub delta: LazyRollingDeltasAmountFromHeight<Sats, SatsSigned, PartsPerMillionSigned64>,
}

impl<C> LazyColumnSpotValuePerBlockWithDeltas<C>
where
    C: ColumnId,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        source: &ReadOnlyColumnarVec<PcoVec<Height, Sats>, C>,
        column: C,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Self {
        let inner =
            LazyColumnSpotValuePerBlock::new(name, version, source, column, indexes, spot_price);
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

impl<C> LazyColumnValuePerBlock<C>
where
    C: ColumnId,
{
    pub(crate) fn new(
        name: &str,
        version: Version,
        sats_source: &ReadOnlyColumnarVec<PcoVec<Height, Sats>, C>,
        cents_source: &ReadOnlyColumnarVec<PcoVec<Height, Cents>, C>,
        column: C,
        indexes: &indexes::Vecs,
    ) -> Self {
        let sats = LazyColumnPerBlock::new(
            &format!("{name}_sats"),
            version,
            sats_source,
            column,
            indexes,
        );
        let btc = LazyPerBlock::from_resolutions::<SatsToBitcoin>(
            name,
            version,
            sats.height.read_only_boxed_clone(),
            &sats.resolutions,
        );
        let cents = LazyColumnPerBlock::new(
            &format!("{name}_cents"),
            version,
            cents_source,
            column,
            indexes,
        );
        let usd = LazyPerBlock::from_resolutions::<CentsUnsignedToDollars>(
            &format!("{name}_usd"),
            version,
            cents.height.read_only_boxed_clone(),
            &cents.resolutions,
        );

        Self {
            btc,
            sats,
            usd,
            cents,
        }
    }
}

#[derive(Clone, Traversable)]
pub struct LazyColumnValuePerBlockCumulativeRolling<C>
where
    C: ColumnId,
{
    pub block: LazyValueBlock,
    pub cumulative: LazyColumnValuePerBlock<C>,
    pub sum: LazyRollingSumsAmountFromHeight,
    pub average: LazyRollingAvgsAmountFromHeight,
}

impl<C> LazyColumnValuePerBlockCumulativeRolling<C>
where
    C: ColumnId,
{
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        name: &str,
        version: Version,
        cumulative_sats: &ReadOnlyColumnarVec<PcoVec<Height, Sats>, C>,
        cumulative_cents: &ReadOnlyColumnarVec<PcoVec<Height, Cents>, C>,
        column: C,
        indexes: &indexes::Vecs,
        cached_starts: &Windows<&CachedWindowStartVec>,
    ) -> Self {
        let cumulative = LazyColumnValuePerBlock::new(
            &format!("{name}_cumulative"),
            version,
            cumulative_sats,
            cumulative_cents,
            column,
            indexes,
        );
        let block = LazyValueBlock::from_cumulative_sources(
            name,
            version,
            &cumulative.sats.height,
            &cumulative.cents.height,
        );
        let sum = LazyRollingSumsAmountFromHeight::new(
            &format!("{name}_sum"),
            version,
            &cumulative.sats.height,
            &cumulative.cents.height,
            cached_starts,
            indexes,
        );
        let average = LazyRollingAvgsAmountFromHeight::new(
            &format!("{name}_average"),
            version,
            &cumulative.sats.height,
            &cumulative.cents.height,
            cached_starts,
            indexes,
        );

        Self {
            block,
            cumulative,
            sum,
            average,
        }
    }
}
