use bitview_distribution::metrics::SupplyBase;
mod vecs;
pub type SupplyTotal<M = vecdb::Rw> =
    bitview_distribution::families::SupplyTotal<bitview_cohort::cohort_group::Utxo, M>;
pub use vecs::SupplyVecs;
