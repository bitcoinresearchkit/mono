use bitview_distribution::metrics::WeightedCohort;
use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use super::DormantVecs;

#[derive(Traversable)]
#[traversable(client = "CointimeCohort")]
pub struct CohortVecs<M: StorageMode = Rw> {
    /// The cohort's stock amounts with each UTXO age range's weighted by its wakefulness, summed and rounded once.
    #[traversable(lead = "Awake coins: each counted by how much of its holding time has been spent, its age band's wakefulness.")]
    pub awake: WeightedCohort<M>,
    pub dormant: DormantVecs,
}
