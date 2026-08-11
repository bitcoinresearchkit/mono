use std::ops::AddAssign;

use brk_cohort::{
    AgeRange, AgeRangeId, AmountRange, AmountRangeId, ByEntry, ByEpoch, Class, ClassId, EntryId,
    EpochId, Filter, OVER_AGE_FILTERS, OVER_AMOUNT_FILTERS, OverAge, OverAmount, SpendableType,
    SpendableTypeId, TERM_FILTERS, UNDER_AGE_FILTERS, UNDER_AMOUNT_FILTERS, UTXOAggregate,
    UnderAge, UnderAmount,
};
use vecdb::{ColumnId, VecValue};

use super::UTXOAggregateRows;

#[derive(Clone, Default)]
pub(crate) struct UTXORows<T> {
    pub age_range: AgeRange<T>,
    pub epoch: ByEpoch<T>,
    pub class: Class<T>,
    pub entry: ByEntry<T>,
    pub amount_range: AmountRange<T>,
    pub type_: SpendableType<T>,
}

impl<T> UTXORows<T> {
    pub(crate) fn map<U>(&self, mut map: impl FnMut(&T) -> U) -> UTXORows<U> {
        UTXORows {
            age_range: AgeRange::from_fn(|id| map(id.select(&self.age_range))),
            epoch: ByEpoch::from_fn(|id| map(id.select(&self.epoch))),
            class: Class::from_fn(|id| map(id.select(&self.class))),
            entry: ByEntry::from_fn(|id| map(id.select(&self.entry))),
            amount_range: AmountRange::from_fn(|id| map(id.select(&self.amount_range))),
            type_: SpendableType::from_fn(|id| map(id.select(&self.type_))),
        }
    }

    pub(crate) fn aggregate(&self) -> UTXOAggregateRows<T>
    where
        T: AddAssign + Clone + Default,
    {
        UTXOAggregateRows {
            aggregate: UTXOAggregate {
                all: Self::sum(self.age_range.iter()),
                sth: self.sum_age(&TERM_FILTERS.short),
                lth: self.sum_age(&TERM_FILTERS.long),
            },
            under_age: UnderAge::from_fn(|id| self.sum_age(id.select(&UNDER_AGE_FILTERS))),
            over_age: OverAge::from_fn(|id| self.sum_age(id.select(&OVER_AGE_FILTERS))),
            under_amount: UnderAmount::from_fn(|id| {
                self.sum_amount(id.select(&UNDER_AMOUNT_FILTERS))
            }),
            over_amount: OverAmount::from_fn(|id| self.sum_amount(id.select(&OVER_AMOUNT_FILTERS))),
        }
    }

    fn sum_age(&self, filter: &Filter) -> T
    where
        T: AddAssign + Clone + Default,
    {
        Self::sum(AgeRangeId::included_by(filter).map(|column| column.select(&self.age_range)))
    }

    fn sum_amount(&self, filter: &Filter) -> T
    where
        T: AddAssign + Clone + Default,
    {
        Self::sum(
            AmountRangeId::included_by(filter).map(|column| column.select(&self.amount_range)),
        )
    }

    fn sum<'a>(values: impl Iterator<Item = &'a T>) -> T
    where
        T: AddAssign + Clone + Default + 'a,
    {
        values.fold(T::default(), |mut total, value| {
            total += value.clone();
            total
        })
    }
}

impl<T> AddAssign for UTXORows<T>
where
    T: VecValue + AddAssign + Copy,
{
    fn add_assign(&mut self, rhs: Self) {
        for id in AgeRangeId::ALL {
            *id.get_mut(&mut self.age_range) += *id.get(&rhs.age_range);
        }
        for id in EpochId::ALL {
            *id.get_mut(&mut self.epoch) += *id.get(&rhs.epoch);
        }
        for id in ClassId::ALL {
            *id.get_mut(&mut self.class) += *id.get(&rhs.class);
        }
        for id in EntryId::ALL {
            *id.get_mut(&mut self.entry) += *id.get(&rhs.entry);
        }
        for id in AmountRangeId::ALL {
            *id.get_mut(&mut self.amount_range) += *id.get(&rhs.amount_range);
        }
        for id in SpendableTypeId::ALL {
            *id.get_mut(&mut self.type_) += *id.get(&rhs.type_);
        }
    }
}
