pub mod families;
pub mod metrics;
pub mod state;

pub mod readers;
mod realized_caps;
pub mod replay;
pub mod walk;
pub use realized_caps::RealizedCaps;

mod all_chain_sources;
pub use all_chain_sources::AllChainSources;

mod weighted_cohort_sources;
pub use weighted_cohort_sources::WeightedCohortSources;

use brk_types::{Cents, Height};
use vecdb::ReadableVec;

/// Exact, reversible cost-basis sums use zero contributions from unpriced coins.
fn read_basis_prices(
    spot: &(impl ReadableVec<Height, Cents> + ?Sized),
    end: usize,
    prices: &mut Vec<Cents>,
) {
    if bitview_vecs::first_price_height().is_some() {
        spot.read_into_at(prices.len(), end, prices);
    } else {
        prices.resize(end, Cents::ZERO);
    }
}
