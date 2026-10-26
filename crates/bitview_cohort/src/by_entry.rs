#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::CohortName;

impl EntryPrice {
    #[inline]
    pub const fn from_is_discount(is_discount: bool) -> Self {
        if is_discount {
            Self::Discount
        } else {
            Self::Premium
        }
    }
}

pub const ENTRY_NAMES: ByEntry<CohortName> = ByEntry {
    discount: CohortName::new("discount", "Discount", "Discount Coins"),
    premium: CohortName::new("premium", "Premium", "Premium Coins"),
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct ByEntry<T> {
    /// Uses UTXOs created when spot price was at or below the previous block's
    /// all-chain capitalized price, the mean creation price of all unspent
    /// outputs weighted by each output's creation-date USD value. An undefined
    /// capitalized price (no priced capital yet) also selects this cohort. Membership is set at
    /// creation and kept until the output is spent.
    #[cfg_attr(feature = "storage", traversable(lead = "Coins that last moved at or below the market's average cost: bought at a discount."))]
    pub discount: T,
    /// Uses UTXOs created when spot price was above the previous block's all-chain
    /// capitalized price, the mean creation price of all unspent outputs
    /// weighted by each output's creation-date USD value. Membership is set at creation and kept
    /// until the output is spent.
    #[cfg_attr(feature = "storage", traversable(lead = "Coins that last moved above the market's average cost: bought at a premium."))]
    pub premium: T,
}

define_cohort_id!(
    EntryPrice for ByEntry {
        Discount => discount,
        Premium => premium,
    }
);

impl<T> ByEntry<T> {
    pub fn get_mut(&mut self, entry: EntryPrice) -> &mut T {
        entry.select_mut(self)
    }
}
