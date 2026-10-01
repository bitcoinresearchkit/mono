use bitview_cohort::{AgeAggregate, AgeRangeId};
use bitview_traversable::Traversable;
use brk_types::{Cents, CentsCompact, Sats};

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutoffs_keep_their_exact_age_boundaries() {
        let entries = [
            (AgeRangeId::From3MTo4M, 100, 1),
            (AgeRangeId::From4MTo5M, 200, 3),
            (AgeRangeId::From5MTo6M, 300, 4),
            (AgeRangeId::From6MTo9M, 400, 2),
            (AgeRangeId::Under1H, 500, 0),
        ]
        .map(|(age, price, sats)| (age, CentsCompact::new(price), Sats::new(sats)));
        let bounds = PriceBounds::from_age_entries(entries);
        assert_eq!(bounds.under_4m.max, Cents::new(100));
        assert_eq!(bounds.sth.max, Cents::new(200));
        assert_eq!(bounds.under_6m.max, Cents::new(300));
        assert_eq!(bounds.all.min, Cents::new(100));
        assert_eq!(bounds.over_4m.min, Cents::new(200));
        assert_eq!(bounds.lth.min, Cents::new(300));
        assert_eq!(bounds.over_6m.min, Cents::new(400));
    }
}
