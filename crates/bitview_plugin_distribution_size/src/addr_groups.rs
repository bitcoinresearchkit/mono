use bitview_traversable::Traversable;

use crate::groups::SizeGroups;
use bitview_cohort::{AmountRange, CohortContext, CohortId};

/// UTXO groups plus groups defined by the controlling address's balance.
///
/// These are independent views, not a cross-product. `A` can own the
/// address-balance storage when its UTXO counterpart is stored separately.
#[derive(Clone, Traversable)]
pub struct SizeAndAddrGroups<T: Clone, A = AmountRange<T>> {
    #[traversable(flatten)]
    pub utxo: SizeGroups<T>,
    /// Groups addresses by their balance, not individual output value.
    pub addr_balance: A,
}

impl<T: Clone> SizeAndAddrGroups<T> {
    pub fn map_with_id<U: Clone>(
        &self,
        mut map: impl FnMut(CohortContext, CohortId, &T) -> U,
    ) -> SizeAndAddrGroups<U> {
        SizeAndAddrGroups {
            utxo: self
                .utxo
                .map_with_id(|id, value| map(CohortContext::Utxo, id, value)),
            addr_balance: AmountRange::from_fn(|id| {
                map(
                    CohortContext::Addr,
                    id.cohort(),
                    id.select(&self.addr_balance),
                )
            }),
        }
    }
}
