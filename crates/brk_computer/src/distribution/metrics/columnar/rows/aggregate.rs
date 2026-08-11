use brk_cohort::{OverAge, OverAmount, UTXOAggregate, UnderAge, UnderAmount};

#[derive(Clone, Default)]
pub struct UTXOAggregateRows<T> {
    pub aggregate: UTXOAggregate<T>,
    pub under_age: UnderAge<T>,
    pub over_age: OverAge<T>,
    pub under_amount: UnderAmount<T>,
    pub over_amount: OverAmount<T>,
}

impl<T> UTXOAggregateRows<T> {
    pub fn map<U>(&self, mut map: impl FnMut(&T) -> U) -> UTXOAggregateRows<U> {
        UTXOAggregateRows {
            aggregate: self.aggregate.map(&mut map),
            under_age: UnderAge::from_fn(|id| map(id.select(&self.under_age))),
            over_age: OverAge::from_fn(|id| map(id.select(&self.over_age))),
            under_amount: UnderAmount::from_fn(|id| map(id.select(&self.under_amount))),
            over_amount: OverAmount::from_fn(|id| map(id.select(&self.over_amount))),
        }
    }
}
