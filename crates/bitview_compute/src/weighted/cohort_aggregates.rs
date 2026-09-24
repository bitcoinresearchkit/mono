use bitview_cohort::{AgeRangeId, ByTerm};

use super::WeightedCohortState;

#[derive(Default)]
pub struct WeightedCohortAggregates {
    pub terms: ByTerm<WeightedCohortState>,
    pub under_4m: WeightedCohortState,
    pub under_6m: WeightedCohortState,
    pub over_4m: WeightedCohortState,
    pub over_6m: WeightedCohortState,
}

impl WeightedCohortAggregates {
    pub fn from_fn(mut contribution: impl FnMut(AgeRangeId) -> WeightedCohortState) -> Self {
        let mut aggregates = Self::default();
        for &id in AgeRangeId::ALL {
            let contribution = contribution(id);
            let term = aggregates.terms.get_mut(id.term());
            *term = term.merged(contribution);
            if id >= AgeRangeId::From4MTo5M {
                aggregates.over_4m = aggregates.over_4m.merged(contribution);
            }
            if id >= AgeRangeId::From6MTo9M {
                aggregates.over_6m = aggregates.over_6m.merged(contribution);
            }
            // Preserve youngest-to-oldest accumulation and the term grouping
            // used at each exact cutoff, including floating-point loss ratios.
            if id == AgeRangeId::From3MTo4M {
                aggregates.under_4m = aggregates.terms.short;
            } else if id == AgeRangeId::From5MTo6M {
                aggregates.under_6m = aggregates.terms.short.merged(aggregates.terms.long);
            }
        }
        aggregates
    }
}
