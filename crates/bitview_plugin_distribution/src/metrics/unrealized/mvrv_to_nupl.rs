use brk_types::{PartsPerMillion64, PartsPerMillionSigned32};
use vecdb::UnaryTransform;

pub struct MvrvToNupl;

impl UnaryTransform<PartsPerMillion64, PartsPerMillionSigned32> for MvrvToNupl {
    #[inline(always)]
    fn apply(mvrv: PartsPerMillion64) -> PartsPerMillionSigned32 {
        PartsPerMillionSigned32::from(1.0 - 1.0 / f64::from(mvrv))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nupl_is_derived_from_mvrv() {
        assert_eq!(
            MvrvToNupl::apply(PartsPerMillion64::from(2.0)),
            PartsPerMillionSigned32::from(0.5),
        );
        assert_eq!(
            MvrvToNupl::apply(PartsPerMillion64::from(1.0)),
            PartsPerMillionSigned32::ZERO,
        );
        assert!(MvrvToNupl::apply(PartsPerMillion64::NAN).is_nan());
        assert!(MvrvToNupl::apply(PartsPerMillion64::ZERO).is_nan());
    }
}
