use bitview_compute::{
    CentsUnsignedToDollars, Identity, LazyIndexedVec, LazyPerBlock, SatsToBitcoin, SatsToCents,
};
use bitview_plugin_mappings::Vecs as MappingVecs;
use bitview_traversable::Traversable;
use brk_error::Result;
use brk_types::{Bitcoin, Cents, Dollars, Height, Sats, Version};
use vecdb::{BinaryTransform, CachedBoxedVec, ReadableCloneableVec, ReadableVec, TypedVec};

use crate::DCA_DOLLARS_PER_DAY;

#[derive(Clone, Traversable)]
pub struct DcaStack {
    /// Reported in BTC; one BTC equals 100,000,000 satoshis.
    pub btc: LazyPerBlock<Bitcoin, Sats>,
    /// Reported in satoshis.
    pub sats: LazyPerBlock<Sats>,
    /// Reported in US dollars.
    pub usd: LazyPerBlock<Dollars, Cents>,
    /// Reported in US cents; 100 cents equal one US dollar.
    pub cents: LazyPerBlock<Cents>,
}

impl DcaStack {
    const COST_BASIS_NUMERATOR: f64 = DCA_DOLLARS_PER_DAY * 100.0 * Sats::ONE_BTC_U64 as f64;

    pub fn from_source<V>(
        name: &str,
        version: Version,
        mappings: &MappingVecs,
        source: V,
        spot_price: &CachedBoxedVec<Height, Cents>,
    ) -> Result<Self>
    where
        V: TypedVec<I = Height, T = Sats> + ReadableVec<Height, Sats> + Clone + 'static,
    {
        let sats = LazyPerBlock::from_height_source::<Identity<Sats>>(
            &format!("{name}_sats"),
            version,
            source,
            mappings,
        );
        let btc = LazyPerBlock::from_lazy::<SatsToBitcoin, Sats>(name, version, &sats);
        let cents_source = LazyIndexedVec::new(
            &format!("{name}_cents_source"),
            version,
            sats.height.read_only_boxed_clone(),
            spot_price.clone(),
            |_, sats, spot| SatsToCents::apply(sats, spot),
        );
        let cents = LazyPerBlock::from_height_source::<Identity<Cents>>(
            &format!("{name}_cents"),
            version,
            cents_source,
            mappings,
        );
        let usd = LazyPerBlock::from_lazy::<CentsUnsignedToDollars, Cents>(
            &format!("{name}_usd"),
            version,
            &cents,
        );
        Ok(Self {
            btc,
            sats,
            usd,
            cents,
        })
    }

    #[inline(always)]
    pub fn cost_basis_cents(days: usize, sats: Sats) -> Cents {
        if sats == Sats::ZERO {
            return Cents::NAN;
        }

        let cents = (Self::COST_BASIS_NUMERATOR * days as f64 / f64::from(sats)).round();
        if cents >= u64::MAX as f64 {
            Cents::MAX_FINITE
        } else {
            Cents::from(cents as u64)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DCA_AMOUNT;

    #[test]
    fn cost_basis_cents_matches_typed_formula() {
        assert_eq!(DcaStack::cost_basis_cents(1, Sats::ZERO), Cents::NAN);
        assert_eq!(
            DcaStack::cost_basis_cents(1, Sats::ONE_BTC),
            Cents::from(10_000_u64),
        );
        assert_eq!(
            DcaStack::cost_basis_cents(usize::MAX, Sats::_1),
            Cents::MAX_FINITE,
        );

        for index in 0..1_200_000_u64 {
            let days = index as usize % 6_000 + 1;
            let sats =
                Sats::from(index.wrapping_mul(6_364_136_223_846_793_005) % 2_000_000_000 + 1);
            assert_eq!(
                DcaStack::cost_basis_cents(days, sats),
                Cents::from(DCA_AMOUNT * days / Bitcoin::from(sats)),
            );
        }
    }
}
