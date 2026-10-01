use bitview_cohort::{AmountRange, CohortId, SpendableType};
use std::ops::AddAssign;
#[derive(Clone, Default)]
pub struct UtxoValues<T> {
    pub amount_range: AmountRange<T>,
    pub type_: SpendableType<T>,
}
impl<T> UtxoValues<T> {
    pub fn map<U>(&self, mut f: impl FnMut(&T) -> U) -> UtxoValues<U> {
        UtxoValues {
            amount_range: AmountRange::from_fn(|id| f(id.select(&self.amount_range))),
            type_: SpendableType::from_fn(|id| f(id.select(&self.type_))),
        }
    }
    pub fn get(&self, id: CohortId) -> Option<&T> {
        match id {
            CohortId::Amount(id) => Some(id.select(&self.amount_range)),
            CohortId::Type(id) => Some(self.type_.get(id)),
            _ => None,
        }
    }
}
impl<T: AddAssign + Copy> AddAssign for UtxoValues<T> {
    fn add_assign(&mut self, rhs: Self) {
        for (a, b) in self.amount_range.iter_mut().zip(rhs.amount_range.iter()) {
            *a += *b;
        }
        for (a, b) in self.type_.iter_mut().zip(rhs.type_.iter()) {
            *a += *b;
        }
    }
}
