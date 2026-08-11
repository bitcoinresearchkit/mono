use brk_types::Cents;
use vecdb::{UnaryTransform, unlikely};

pub struct CentsTimesTenths<const V: u16>;

impl<const V: u16> UnaryTransform<Cents, Cents> for CentsTimesTenths<V> {
    #[inline(always)]
    fn apply(cents: Cents) -> Cents {
        if unlikely(cents.is_nan()) {
            Cents::NAN
        } else {
            Cents::from(cents.as_u128() * V as u128 / 10)
        }
    }
}
