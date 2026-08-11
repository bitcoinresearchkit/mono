use brk_types::{CentsSigned, Dollars};
use vecdb::UnaryTransform;

pub struct CentsSignedToDollars;

impl UnaryTransform<CentsSigned, Dollars> for CentsSignedToDollars {
    #[inline(always)]
    fn apply(cents: CentsSigned) -> Dollars {
        cents.into()
    }
}
