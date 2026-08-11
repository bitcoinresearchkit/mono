use brk_types::{Cents, Dollars};
use vecdb::UnaryTransform;

pub struct CentsUnsignedToDollars;

impl UnaryTransform<Cents, Dollars> for CentsUnsignedToDollars {
    #[inline(always)]
    fn apply(cents: Cents) -> Dollars {
        cents.into()
    }
}
