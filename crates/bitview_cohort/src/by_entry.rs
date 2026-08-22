use bitview_traversable::Traversable;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use schemars::JsonSchema;
use serde::Serialize;

use super::{CohortName, Filter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntryPrice {
    Discount,
    Premium,
}

impl EntryPrice {
    #[inline]
    pub const fn from_is_discount(is_discount: bool) -> Self {
        if is_discount {
            Self::Discount
        } else {
            Self::Premium
        }
    }

    #[inline]
    pub const fn is_discount(self) -> bool {
        matches!(self, Self::Discount)
    }
}

pub const ENTRY_FILTERS: ByEntry<Filter> = ByEntry {
    discount: Filter::Entry(EntryPrice::Discount),
    premium: Filter::Entry(EntryPrice::Premium),
};

pub const ENTRY_NAMES: ByEntry<CohortName> = ByEntry {
    discount: CohortName::new("veteran", "Veteran", "Veteran Coins"),
    premium: CohortName::new("rookie", "Rookie", "Rookie Coins"),
};

#[derive(Debug, Default, Clone, Traversable, Serialize, JsonSchema)]
pub struct ByEntry<T> {
    /// Uses UTXOs created when spot price was at or below the then-current
    /// all-chain capitalized price, the mean creation price of all unspent
    /// outputs weighted by each output's creation-date USD value.
    pub discount: T,
    /// Uses UTXOs created when spot price was above the then-current all-chain
    /// capitalized price, the mean creation price of all unspent outputs
    /// weighted by each output's creation-date USD value.
    pub premium: T,
}

define_column_id!(
    EntryId for ByEntry, version = 1 {
        Discount => discount,
        Premium => premium,
    }
);

impl ByEntry<CohortName> {
    pub const fn names() -> &'static Self {
        &ENTRY_NAMES
    }
}

impl<T> ByEntry<T> {
    pub fn new<F>(mut create: F) -> Self
    where
        F: FnMut(Filter, &'static str) -> T,
    {
        let f = ENTRY_FILTERS;
        let n = ENTRY_NAMES;
        Self {
            discount: create(f.discount, n.discount.id),
            premium: create(f.premium, n.premium.id),
        }
    }

    pub fn try_new<F, E>(mut create: F) -> Result<Self, E>
    where
        F: FnMut(Filter, &'static str) -> Result<T, E>,
    {
        let f = ENTRY_FILTERS;
        let n = ENTRY_NAMES;
        Ok(Self {
            discount: create(f.discount, n.discount.id)?,
            premium: create(f.premium, n.premium.id)?,
        })
    }

    pub fn get(&self, entry: EntryPrice) -> &T {
        match entry {
            EntryPrice::Discount => &self.discount,
            EntryPrice::Premium => &self.premium,
        }
    }

    pub fn get_mut(&mut self, entry: EntryPrice) -> &mut T {
        match entry {
            EntryPrice::Discount => &mut self.discount,
            EntryPrice::Premium => &mut self.premium,
        }
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        [&self.discount, &self.premium].into_iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        [&mut self.discount, &mut self.premium].into_iter()
    }

    pub fn par_iter_mut(&mut self) -> impl ParallelIterator<Item = &mut T>
    where
        T: Send + Sync,
    {
        [&mut self.discount, &mut self.premium].into_par_iter()
    }
}
