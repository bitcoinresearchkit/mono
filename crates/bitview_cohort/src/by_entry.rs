#[cfg(feature = "storage")]
use bitview_traversable::Traversable;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{CohortId, CohortName};

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
    discount: CohortName::new("veteran", "Veteran", "Veteran Coins"),
    premium: CohortName::new("rookie", "Rookie", "Rookie Coins"),
};

#[derive(Debug, Default, Clone, Serialize, Deserialize, JsonSchema)]
#[cfg_attr(feature = "storage", derive(Traversable))]
pub struct ByEntry<T> {
    /// Uses UTXOs created when spot price was at or below the previous block's
    /// all-chain capitalized price, the mean creation price of all unspent
    /// outputs weighted by each output's creation-date USD value. A zero
    /// capitalized price also selects this cohort.
    pub discount: T,
    /// Uses UTXOs created when spot price was above the previous block's all-chain
    /// capitalized price, the mean creation price of all unspent outputs
    /// weighted by each output's creation-date USD value.
    pub premium: T,
}

define_cohort_id!(
    EntryPrice for ByEntry {
        Discount => discount,
        Premium => premium,
    }
);

impl<T> ByEntry<T> {
    pub fn new(mut create: impl FnMut(CohortId) -> T) -> Self {
        Self::from_fn(|entry| create(CohortId::Entry(entry)))
    }

    pub fn try_new<E>(mut create: impl FnMut(CohortId) -> Result<T, E>) -> Result<Self, E> {
        Self::try_from_fn(|entry| create(CohortId::Entry(entry)))
    }

    pub fn get(&self, entry: EntryPrice) -> &T {
        entry.select(self)
    }

    pub fn get_mut(&mut self, entry: EntryPrice) -> &mut T {
        entry.select_mut(self)
    }
}
