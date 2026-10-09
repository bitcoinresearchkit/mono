mod collection;
mod price;
type RealizedCapByCohort<M = vecdb::Rw> = bitview_distribution::families::FiatByCohort<
    bitview_cohort::cohort_group::Utxo,
    brk_types::Cents,
    M,
>;
pub use collection::RealizedVecs;
type CumulativeRealizedByCohort<M = vecdb::Rw> =
    bitview_distribution::families::CumulativeRealizedByCohort<
        bitview_cohort::cohort_group::Utxo,
        M,
    >;
use price::RealizedPriceByCohort;
