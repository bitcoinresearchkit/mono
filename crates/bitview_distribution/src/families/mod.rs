//! One cohort's stored series and their public views, shared by the creation (age) and UTXO
//! plugins. Stored names are `{cohort}_{metric}...` (`CohortContext::Utxo`).

mod cumulative_count;
mod cumulative_fiat;
mod cumulative_value;
mod fiat;
mod supply;
mod unspent_output_count;

pub use cumulative_count::CumulativeCount;
pub use cumulative_fiat::CumulativeFiat;
pub use cumulative_value::CumulativeValue;
pub use fiat::Fiat;
pub use supply::Supply;
pub use unspent_output_count::UnspentOutputCount;

use bitview_cohort::{CohortContext, CohortId};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, PcoVecValue};

use bitview_vecs::{CachedSeries, import_cached};

fn import_stored<T: PcoVecValue>(
    db: &Database,
    cohort: CohortId,
    name: &str,
    version: Version,
) -> Result<CachedSeries<Height, T>> {
    import_cached(db, &stored_name(cohort, name), version + Version::TWO)
}

fn stored_name(cohort: CohortId, name: &str) -> String {
    CohortContext::Utxo.metric_name(cohort, name)
}
