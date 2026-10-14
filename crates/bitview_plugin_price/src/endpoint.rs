use bitview_vecs::{
    AggFold, IndexSources, LazyAggVec, LazyIndexes, Price, ops::Convert, shapes::PerResolution,
    with_resolution_fields,
};
use brk_types::{Cents, Dollars, Height, Version};
use rangeindex::RangeMap;
use vecdb::{AnyVec, LazyVec, ReadableBoxedVec, ReadableCloneableVec, ReadableVec, VecIndex};

const COMPUTE_VERSION: Version = Version::ONE;

/// A period's opening (`CLOSE = false`) or closing spot price, read at one height instead of
/// building its candle: the first or last block of a populated period. An empty period carries
/// the previous close, zero before the first price, as its flat candle does.
pub(crate) struct Endpoint<const CLOSE: bool>;

impl<const CLOSE: bool> Endpoint<CLOSE> {
    fn height(first_heights: &[Height], index: usize, price_len: usize) -> Option<usize> {
        let first = first_heights[index].to_usize().min(price_len);
        let end = first_heights
            .get(index + 1)
            .map_or(price_len, |height| height.to_usize().min(price_len));
        let last = end.checked_sub(1)?;
        Some(if CLOSE { last } else { first.min(last) })
    }

    /// One sorted read of the requested periods' heights; adjacent empty periods share one.
    fn read<S: ReadableVec<Height, Cents> + ?Sized>(
        source: &S,
        first_heights: &[Height],
        indices: impl ExactSizeIterator<Item = usize>,
    ) -> impl Iterator<Item = Cents> {
        let price_len = source.visible_len();
        let mut heights = Vec::with_capacity(indices.len());
        let slots: Vec<Option<usize>> = indices
            .map(|index| {
                Self::height(first_heights, index, price_len).map(|height| {
                    if heights.last() != Some(&height) {
                        heights.push(height);
                    }
                    heights.len() - 1
                })
            })
            .collect();
        let prices = source.read_sorted_at(&heights);
        slots
            .into_iter()
            .map(move |slot| slot.map_or(Cents::ZERO, |slot| prices[slot]))
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
        Self::read(source, mapping.as_slice(), from..to).try_fold(init, f)
    }

    fn read_sorted_into<MI: VecIndex, S: ReadableVec<Height, Cents> + ?Sized>(
        source: &S,
        mapping: &RangeMap<Height, MI>,
        indices: &[usize],
        out: &mut Vec<Cents>,
    ) {
        out.extend(Self::read(
            source,
            mapping.as_slice(),
            indices.iter().copied(),
        ));
    }
}

macro_rules! define_endpoint_price {
    (
        periods { $($field:ident: $index:ident => $param:ident,)* }
        epochs { $($epoch:ident: $epoch_index:ident => $epoch_param:ident,)* }
    ) => {
        use bitview_primitives::{$($index,)* $($epoch_index,)*};

        /// Opening or closing cents per resolution.
        pub(crate) type EndpointCents<const CLOSE: bool> = PerResolution<
            $(LazyAggVec<$index, Cents, Height, Cents, Endpoint<CLOSE>>,)*
            $(LazyAggVec<$epoch_index, Cents, Height, Cents, Endpoint<CLOSE>>,)*
        >;

        /// The opening or closing price per resolution: cents read from `prices`, lazy USD.
        pub(crate) fn endpoint_price<const CLOSE: bool>(
            name: &str,
            version: Version,
            indexes: &IndexSources,
            prices: &ReadableBoxedVec<Height, Cents>,
        ) -> Price<EndpointCents<CLOSE>, LazyIndexes<Dollars, Cents>> {
            let v = version + COMPUTE_VERSION;
            let cents_name = format!("{name}_cents");
            let cents: EndpointCents<CLOSE> = PerResolution {
                $($field: LazyAggVec::new(
                    &cents_name, v + indexes.first_height.$field.version(), prices.clone(),
                    indexes.first_height.$field.mapping().clone(),
                ),)*
                $($epoch: LazyAggVec::new(
                    &cents_name, v + indexes.first_height.$epoch.version(), prices.clone(),
                    indexes.first_height.$epoch.mapping().clone(),
                ),)*
            };
            let usd = LazyIndexes(PerResolution {
                $($field: LazyVec::transformed::<Convert>(name, version, cents.$field.read_only_boxed_clone()),)*
                $($epoch: LazyVec::transformed::<Convert>(name, version, cents.$epoch.read_only_boxed_clone()),)*
            });
            Price { usd, cents }
        }
    };
}

with_resolution_fields!(define_endpoint_price);
