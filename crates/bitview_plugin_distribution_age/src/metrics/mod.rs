mod activity;
mod cohorts;

mod outputs;

mod realized;

mod supply;
mod unrealized;

pub use activity::ActivityVecs;
use bitview_cohort::cohort_group::Creation;
use bitview_vecs::CumulativeCohortSources;
use vecdb::Rw;

pub type CumulativeCreationSources<T, M = Rw> = CumulativeCohortSources<Creation, T, M>;
pub use cohorts::CohortMetrics;

pub use outputs::OutputsVecs;

pub use realized::RealizedBlockData;
pub use realized::RealizedVecs;
pub use unrealized::UnrealizedVecs;

pub use supply::SupplyVecs;
