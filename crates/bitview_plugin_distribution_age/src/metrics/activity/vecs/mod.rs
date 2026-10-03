mod coindays_destroyed;
mod collection;
mod core_cumulative_value;

pub use coindays_destroyed::CoindaysDestroyedByCohort;
pub use collection::ActivityVecs;
pub use core_cumulative_value::CoreCumulativeValueByCohort;
pub type CumulativeValueByCohort<M = vecdb::Rw> =
    bitview_plugin_distribution_common::families::CumulativeValueByCohort<
        bitview_cohort::cohort_group::Creation,
        M,
    >;
