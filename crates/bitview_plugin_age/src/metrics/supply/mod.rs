use bitview_distribution::metrics::SupplyBase;
mod vecs;

pub type SupplyByCohort<M = vecdb::Rw> =
    bitview_distribution::families::SupplyByCohort<bitview_cohort::cohort_group::Creation, M>;
pub type SupplyTotal<M = vecdb::Rw> =
    bitview_distribution::families::SupplyTotal<bitview_cohort::cohort_group::Creation, M>;
pub use vecs::SupplyVecs;
