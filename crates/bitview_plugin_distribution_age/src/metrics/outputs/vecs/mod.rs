mod collection;

pub use collection::OutputsVecs;
pub type SpentOutputCount<M = vecdb::Rw> =
    bitview_plugin_distribution_common::families::SpentOutputCount<
        bitview_cohort::cohort_group::Creation,
        M,
    >;
pub type UnspentOutputCount<M = vecdb::Rw> =
    bitview_plugin_distribution_common::families::UnspentOutputCount<
        bitview_cohort::cohort_group::Creation,
        M,
    >;
