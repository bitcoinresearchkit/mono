use bitview_cohort::{AgeAggregate, AgeRangeId, ByTerm};

use super::WeightedCohortState;

/// Merges each age range's weighted contribution into every age aggregate.
pub fn weighted_age_aggregates(
    mut contribution: impl FnMut(AgeRangeId) -> WeightedCohortState,
) -> AgeAggregate<WeightedCohortState> {
    let mut terms = ByTerm::<WeightedCohortState>::default();
    let mut aggregates = AgeAggregate::<WeightedCohortState>::default();
    for &id in AgeRangeId::ALL {
        let contribution = contribution(id);
        let term = terms.get_mut(id.term());
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
            aggregates.under_4m = terms.short;
        } else if id == AgeRangeId::From5MTo6M {
            aggregates.under_6m = terms.short.merged(terms.long);
        }
    }
    aggregates.all = terms.short.merged(terms.long);
    aggregates.sth = terms.short;
    aggregates.lth = terms.long;
    aggregates
}
