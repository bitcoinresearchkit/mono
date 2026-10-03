use bitview_plugin_distribution_common::metrics::SupplyBase;
mod by_cohort;

mod vecs;

pub use by_cohort::SupplyByCohort;
pub type SupplyTotal<M = vecdb::Rw> = bitview_plugin_distribution_common::families::SupplyTotal<
    bitview_cohort::cohort_group::Creation,
    M,
>;
pub use vecs::SupplyVecs;
