#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::CohortName;

impl EntryPrice {
    #[inline]
    pub const fn from_is_discount(is_discount: bool) -> Self {
        if is_discount {
            Self::Veteran
        } else {
            Self::Rookie
        }
    }
}

pub const ENTRY_NAMES: ByEntry<CohortName> = ByEntry {
    veteran: CohortName::new("veteran", "Veteran", "Veteran Coins"),
    rookie: CohortName::new("rookie", "Rookie", "Rookie Coins"),
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct ByEntry<T> {
    /// Uses UTXOs created when spot price was at or below the previous block's
    /// all-chain capitalized price, the mean creation price of all unspent
    /// outputs weighted by each output's creation-date USD value. A zero
    /// capitalized price also selects this cohort.
    pub veteran: T,
    /// Uses UTXOs created when spot price was above the previous block's all-chain
    /// capitalized price, the mean creation price of all unspent outputs
    /// weighted by each output's creation-date USD value.
    pub rookie: T,
}

define_cohort_id!(
    EntryPrice for ByEntry {
        Veteran => veteran,
        Rookie => rookie,
    }
);

impl<T> ByEntry<T> {
    pub fn get_mut(&mut self, entry: EntryPrice) -> &mut T {
        entry.select_mut(self)
    }
}
