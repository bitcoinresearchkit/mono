use brk_types::PartsPerMillionSigned64;
use vecdb::UnaryTransform;

pub struct Cagr<const YEARS: u8>;

impl<const YEARS: u8> UnaryTransform<PartsPerMillionSigned64, PartsPerMillionSigned64>
    for Cagr<YEARS>
{
    #[inline(always)]
    fn apply(value: PartsPerMillionSigned64) -> PartsPerMillionSigned64 {
        let ratio = f64::from(value) + 1.0;
        let annualized = match YEARS {
            2 => ratio.sqrt(),
            3 if ratio >= 0.0 => ratio.cbrt(),
            4 => ratio.sqrt().sqrt(),
            6 => ratio.cbrt().sqrt(),
            8 => ratio.sqrt().sqrt().sqrt(),
            _ => ratio.powf(1.0 / YEARS as f64),
        };
        PartsPerMillionSigned64::from(annualized - 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_matches_powf<const YEARS: u8>() {
        for index in 0..1_200_000_u64 {
            let ppm = index.wrapping_mul(6_364_136_223_846_793_005) % 20_000_001;
            let value = PartsPerMillionSigned64::from((ppm as f64 - 1_000_000.0) / 1_000_000.0);
            let expected = PartsPerMillionSigned64::from(
                (f64::from(value) + 1.0).powf(1.0 / YEARS as f64) - 1.0,
            );
            assert_eq!(Cagr::<YEARS>::apply(value), expected);
        }
    }

    #[test]
    fn optimized_roots_match_powf() {
        assert_matches_powf::<2>();
        assert_matches_powf::<3>();
        assert_matches_powf::<4>();
        assert_matches_powf::<6>();
        assert_matches_powf::<8>();

        let invalid = PartsPerMillionSigned64::from(-2.0);
        let expected =
            PartsPerMillionSigned64::from((f64::from(invalid) + 1.0).powf(1.0 / 3.0) - 1.0);
        assert_eq!(Cagr::<3>::apply(invalid), expected);
    }
}
