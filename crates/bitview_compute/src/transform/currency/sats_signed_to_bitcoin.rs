use brk_types::{Bitcoin, SatsSigned};
use vecdb::UnaryTransform;

pub struct SatsSignedToBitcoin;

impl UnaryTransform<SatsSigned, Bitcoin> for SatsSignedToBitcoin {
    #[inline(always)]
    fn apply(sats: SatsSigned) -> Bitcoin {
        Bitcoin::from(sats)
    }
}
