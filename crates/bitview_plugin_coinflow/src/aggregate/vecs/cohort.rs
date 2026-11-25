use bitview_distribution::metrics::WeightedCohort;
use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use super::ImmobileVecs;

#[derive(Traversable)]
#[traversable(client = "CoinflowCohort")]
pub struct CohortVecs<M: StorageMode = Rw> {
    /// The cohort's stock amounts with each UTXO age range's weighted by its mobility, summed and rounded once.
    #[traversable(lead = "Mobile coins: each counted by how likely it is to be spent, its age band's mobility.")]
    pub mobile: WeightedCohort<M>,
    pub immobile: ImmobileVecs,
}
