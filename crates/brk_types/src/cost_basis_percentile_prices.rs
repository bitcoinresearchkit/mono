use crate::{Cents, PERCENTILES_LEN};

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CostBasisPercentilePrices {
    /// Price percentiles weighted by satoshis.
    pub per_coin: [Cents; PERCENTILES_LEN],
    /// Price percentiles weighted by acquisition value.
    pub per_dollar: [Cents; PERCENTILES_LEN],
}
