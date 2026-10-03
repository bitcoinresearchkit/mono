use bitview_cohort::{AgeAggregate, AgeRangeId};
use bitview_primitives::CentsCompact;
use bitview_traversable::Traversable;
use brk_types::{Cents, Sats};

#[derive(Clone, Copy, Traversable)]
pub struct PriceBounds<T> {
    /// Lowest occupied price bucket.
    pub min: T,
    /// Highest occupied price bucket.
    pub max: T,
}

impl Default for PriceBounds<Cents> {
    fn default() -> Self {
        Self {
            min: Cents::NAN,
            max: Cents::NAN,
        }
    }
}

impl PriceBounds<Cents> {
    pub fn include(&mut self, price: Cents) {
        if self.min.is_nan() || price < self.min {
            self.min = price;
        }
        if self.max.is_nan() || price > self.max {
            self.max = price;
        }
    }
}

impl PriceBounds<Cents> {
    pub fn from_age_entries(
        entries: impl IntoIterator<Item = (AgeRangeId, CentsCompact, Sats)>,
    ) -> AgeAggregate<Self> {
        let mut bounds = AgeAggregate::<Self>::default();
        for (age, price, sats) in entries {
            if sats != Sats::ZERO {
                for cohort in bounds.containing_mut(age) {
                    cohort.include(Cents::from(price));
                }
            }
        }
        bounds
    }
}
