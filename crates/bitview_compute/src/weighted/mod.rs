mod capitalized_price;
mod cohort_aggregates;
mod cohort_state;
mod ratio;

pub use capitalized_price::WeightedCapitalizedPrice;
pub use cohort_aggregates::weighted_age_aggregates;
pub use cohort_state::WeightedCohortState;
pub use ratio::WeightedRatio;

use bitview_primitives::BoundedRatio;

/// A weight its range's history cannot define yet (no coin-days in the range) moves nothing: the
/// coins it would split stay on the unweighted side (dormant, immobile) rather than on neither.
/// The published weight itself stays undefined.
#[inline]
pub(crate) fn defined_weight(weight: BoundedRatio) -> BoundedRatio {
    if weight.is_nan() {
        BoundedRatio::ZERO
    } else {
        weight
    }
}
