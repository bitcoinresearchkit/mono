use bitview_vecs::{
    AggFold, AggResolutions, IndexSources, LazyIndexes, Price, read_sorted_optional,
};
use brk_types::{Cents, Dollars, Height, Version};
use rangeindex::{Pages, RangeMap};
use vecdb::{Convert, ReadableBoxedVec, ReadableVec, VecIndex};

use crate::FIRST_PRICE_HEIGHT;

/// Two: a period holding the first price opens at it.
const COMPUTE_VERSION: Version = Version::TWO;

/// A period's opening (`CLOSE = false`) or closing spot price, read at one height instead of
/// building its candle: the first priced or last block of a populated period. An empty period
/// carries the previous close, zero before the first price, as its flat candle does.
pub(crate) struct Endpoint<const CLOSE: bool>;

impl<const CLOSE: bool> Endpoint<CLOSE> {
    fn height(first_heights: &Pages<Height>, index: usize, price_len: usize) -> Option<usize> {
        // The zero placeholders before the first price are no prices.
        let first = first_heights[index].to_usize().max(FIRST_PRICE_HEIGHT);
        let end = first_heights
            .get(index + 1)
            .map_or(price_len, |height| height.to_usize().min(price_len));
        let last = end.checked_sub(1)?;
        Some(if CLOSE { last } else { first.min(last) })
    }

    /// One sorted read of the requested periods' heights; adjacent empty periods share one.
    fn read<S: ReadableVec<Height, Cents> + ?Sized>(
        source: &S,
        first_heights: &Pages<Height>,
        indices: impl ExactSizeIterator<Item = usize>,
    ) -> impl Iterator<Item = Cents> {
        let price_len = source.visible_len();
        read_sorted_optional(
            source,
            indices.map(move |index| Self::height(first_heights, index, price_len)),
            true,
        )
        .map(|price| price.unwrap_or(Cents::ZERO))
    }
}

impl<const CLOSE: bool> AggFold<Cents, Height, Cents> for Endpoint<CLOSE> {
    fn try_fold<
        MI: VecIndex,
        S: ReadableVec<Height, Cents> + ?Sized,
        B,
        E,
        F: FnMut(B, Cents) -> Result<B, E>,
    >(
        source: &S,
        mapping: &RangeMap<Height, MI>,
        from: usize,
        to: usize,
        init: B,
        f: F,
    ) -> Result<B, E> {
        Self::read(source, mapping.starts(), from..to).try_fold(init, f)
    }

    fn read_sorted_into<MI: VecIndex, S: ReadableVec<Height, Cents> + ?Sized>(
        source: &S,
        mapping: &RangeMap<Height, MI>,
        indices: &[usize],
        out: &mut Vec<Cents>,
    ) {
        out.extend(Self::read(
            source,
            mapping.starts(),
            indices.iter().copied(),
        ));
    }
}

/// Opening or closing cents per resolution.
type EndpointCents<const CLOSE: bool> = AggResolutions<Cents, Cents, Endpoint<CLOSE>>;
/// Opening or closing price per resolution: cents, with a lazy USD view.
pub(crate) type EndpointPrice<const CLOSE: bool> =
    Price<EndpointCents<CLOSE>, LazyIndexes<Dollars, Cents>>;

/// The opening or closing price per resolution: cents read from `prices`, lazy USD.
pub(crate) fn endpoint_price<const CLOSE: bool>(
    name: &str,
    version: Version,
    indexes: &IndexSources,
    prices: &ReadableBoxedVec<Height, Cents>,
) -> EndpointPrice<CLOSE> {
    let cents = EndpointCents::<CLOSE>::aggregate(
        &format!("{name}_cents"),
        version + COMPUTE_VERSION,
        prices,
        indexes,
    );
    let usd = LazyIndexes::from_resolutions::<Convert>(name, version, &cents);
    Price { usd, cents }
}
