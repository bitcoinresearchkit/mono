use bitview_plugin_distribution_common::metrics::SupplyBase;
mod vecs;
pub type SupplyTotal<M = vecdb::Rw> = bitview_plugin_distribution_common::families::SupplyTotal<
    bitview_cohort::cohort_group::Utxo,
    M,
>;
pub use vecs::SupplyVecs;
