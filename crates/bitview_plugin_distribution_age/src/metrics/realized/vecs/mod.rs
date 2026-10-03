mod collection;
mod cumulative_net;

mod value_destroyed;

pub type RealizedCapByCohort<M = vecdb::Rw> =
    bitview_plugin_distribution_common::families::RealizedCapByCohort<
        bitview_cohort::cohort_group::Creation,
        M,
    >;
pub use collection::RealizedVecs;
pub type CumulativeRealizedByCohort<M = vecdb::Rw> =
    bitview_plugin_distribution_common::families::CumulativeRealizedByCohort<
        bitview_cohort::cohort_group::Creation,
        M,
    >;
pub use cumulative_net::CumulativeNetRealizedByCohort;
pub use value_destroyed::CumulativeValueDestroyedByCohort;
