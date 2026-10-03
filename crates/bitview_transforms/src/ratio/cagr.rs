use bitview_primitives::PartsPerMillionSigned64;
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
