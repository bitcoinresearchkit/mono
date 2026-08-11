use brk_types::{Cents, Dollars};
use vecdb::UnaryTransform;

pub struct NegCentsUnsignedToDollars;

impl UnaryTransform<Cents, Dollars> for NegCentsUnsignedToDollars {
    #[inline(always)]
    fn apply(cents: Cents) -> Dollars {
        -Dollars::from(cents)
    }
}
