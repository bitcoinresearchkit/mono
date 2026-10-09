mod collection;
mod cumulative_net;

mod value_destroyed;

pub type RealizedCapByCohort<M = vecdb::Rw> = bitview_distribution::families::FiatByCohort<
    bitview_cohort::cohort_group::Creation,
    brk_types::Cents,
    M,
>;
pub use collection::RealizedVecs;
pub type CumulativeRealizedByCohort<M = vecdb::Rw> =
    bitview_distribution::families::CumulativeRealizedByCohort<
        bitview_cohort::cohort_group::Creation,
        M,
    >;
pub use cumulative_net::CumulativeNetRealizedByCohort;
pub use value_destroyed::CumulativeValueDestroyedByCohort;
