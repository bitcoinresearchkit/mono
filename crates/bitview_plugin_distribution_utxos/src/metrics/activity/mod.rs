pub type CumulativeValueByCohort<M = vecdb::Rw> =
    bitview_plugin_distribution_common::families::CumulativeValueByCohort<
        bitview_cohort::cohort_group::Utxo,
        M,
    >;
mod collection;
pub use collection::ActivityVecs;
