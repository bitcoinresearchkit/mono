use bitview_plugin_distribution_common::metrics::SupplyBase;
mod vecs;

pub type SupplyByCohort<M = vecdb::Rw> =
    bitview_plugin_distribution_common::families::SupplyByCohort<
        bitview_cohort::cohort_group::Creation,
        M,
    >;
pub type SupplyTotal<M = vecdb::Rw> = bitview_plugin_distribution_common::families::SupplyTotal<
    bitview_cohort::cohort_group::Creation,
    M,
>;
pub use vecs::SupplyVecs;
