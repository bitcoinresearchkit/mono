pub type CumulativeValueByCohort<M = vecdb::Rw> =
    bitview_distribution::families::CumulativeValueByCohort<bitview_cohort::cohort_group::Utxo, M>;
mod collection;
pub use collection::ActivityVecs;
