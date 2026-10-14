use bitview_primitives::OHLCCents;
use brk_types::Cents;
use vecdb::UnaryTransform;

/// One price of an OHLC candle.
macro_rules! ohlc_field {
    ($($name:ident => $field:ident),+ $(,)?) => {$(
        pub struct $name;

        impl UnaryTransform<OHLCCents, Cents> for $name {
            #[inline(always)]
            fn apply(cents: OHLCCents) -> Cents {
                *cents.$field
            }
        }
    )+};
}

ohlc_field!(
    OhlcCentsToOpenCents => open,
    OhlcCentsToHighCents => high,
    OhlcCentsToLowCents => low,
    OhlcCentsToCloseCents => close,
);
