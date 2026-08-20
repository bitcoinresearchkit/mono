use bitview_cohort::UTXOAggregate;
use derive_more::Deref;

use super::WeightedPair;

#[derive(Deref)]
pub struct WeightedUrpdNames(UTXOAggregate<WeightedPair<String>>);

impl WeightedUrpdNames {
    pub fn new(names: UTXOAggregate<WeightedPair<String>>) -> Self {
        Self(names)
    }
}
