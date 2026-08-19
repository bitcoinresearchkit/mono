use std::marker::PhantomData;

use brk_types::{Bytes, Cents, CentsSigned, Dollars, Sats, StoredF32, StoredU64};
use vecdb::{BinaryTransform, unlikely};

use crate::{FixedRatio, NumericValue};

pub struct RatioU64<P>(PhantomData<P>);

impl<P: FixedRatio> BinaryTransform<StoredU64, StoredU64, P> for RatioU64<P> {
    #[inline(always)]
    fn apply(numerator: StoredU64, denominator: StoredU64) -> P {
        if *denominator > 0 {
            P::from(*numerator as f64 / *denominator as f64)
        } else {
            P::default()
        }
    }
}

pub struct RatioBytes<P>(PhantomData<P>);

impl<P: FixedRatio, D: NumericValue> BinaryTransform<Bytes, D, P> for RatioBytes<P> {
    #[inline(always)]
    fn apply(numerator: Bytes, denominator: D) -> P {
        let denominator: f64 = denominator.into();
        if denominator > 0.0 {
            P::from(f64::from(numerator) / denominator)
        } else {
            P::default()
        }
    }
}

pub struct RatioSats<P>(PhantomData<P>);

impl<P: FixedRatio> BinaryTransform<Sats, Sats, P> for RatioSats<P> {
    #[inline(always)]
    fn apply(numerator: Sats, denominator: Sats) -> P {
        if *denominator > 0 {
            P::from(*numerator as f64 / *denominator as f64)
        } else {
            P::default()
        }
    }
}

pub struct RatioCents<P>(PhantomData<P>);

impl<P: FixedRatio> BinaryTransform<Cents, Cents, P> for RatioCents<P> {
    #[inline(always)]
    fn apply(numerator: Cents, denominator: Cents) -> P {
        let denominator = f64::from(denominator);
        if unlikely(denominator == 0.0) {
            P::default()
        } else {
            P::from(f64::from(numerator) / denominator)
        }
    }
}

pub struct RatioDollars<P>(PhantomData<P>);

impl<P: FixedRatio> BinaryTransform<Dollars, Dollars, P> for RatioDollars<P> {
    #[inline(always)]
    fn apply(numerator: Dollars, denominator: Dollars) -> P {
        let ratio = f64::from(numerator) / f64::from(denominator);
        if ratio.is_finite() {
            P::from(ratio)
        } else {
            P::default()
        }
    }
}

pub struct RatioCentsSignedCents<P>(PhantomData<P>);

impl<P: FixedRatio> BinaryTransform<CentsSigned, Cents, P> for RatioCentsSignedCents<P> {
    #[inline(always)]
    fn apply(numerator: CentsSigned, denominator: Cents) -> P {
        let denominator = f64::from(denominator);
        if unlikely(denominator == 0.0) {
            P::default()
        } else {
            P::from(numerator.inner() as f64 / denominator)
        }
    }
}

pub struct RatioDiffF32<P>(PhantomData<P>);

impl<P: FixedRatio> BinaryTransform<StoredF32, StoredF32, P> for RatioDiffF32<P> {
    #[inline(always)]
    fn apply(value: StoredF32, base: StoredF32) -> P {
        if base.is_nan() || *base == 0.0 {
            P::default()
        } else {
            P::from((*value / *base - 1.0) as f64)
        }
    }
}

pub struct RatioDiffDollars<P>(PhantomData<P>);

impl<P: FixedRatio> BinaryTransform<Dollars, Dollars, P> for RatioDiffDollars<P> {
    #[inline(always)]
    fn apply(close: Dollars, base: Dollars) -> P {
        let base = f64::from(base);
        if base == 0.0 {
            P::default()
        } else {
            P::from(f64::from(close) / base - 1.0)
        }
    }
}

pub struct RatioDiffCents<P>(PhantomData<P>);

impl<P: FixedRatio> BinaryTransform<Cents, Cents, P> for RatioDiffCents<P> {
    #[inline(always)]
    fn apply(close: Cents, base: Cents) -> P {
        let base = f64::from(base);
        if base == 0.0 {
            P::default()
        } else {
            P::from(f64::from(close) / base - 1.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use brk_types::PartsPerMillion32;

    use super::*;

    #[test]
    fn cents_ratios_propagate_nan() {
        assert!(RatioCents::<PartsPerMillion32>::apply(Cents::NAN, Cents::new(100)).is_nan());
        assert!(RatioCents::<PartsPerMillion32>::apply(Cents::new(100), Cents::NAN).is_nan());
        assert!(
            RatioCentsSignedCents::<PartsPerMillion32>::apply(CentsSigned::new(100), Cents::NAN,)
                .is_nan()
        );
    }
}
