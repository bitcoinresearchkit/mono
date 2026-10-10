//! One cohort's stored series and their public views, shared by the cohort plugins. Each takes
//! the cohort's series name (`utxos_1d_to_1w_old_supply`, `balance_1btc_to_10btc_supply`); its
//! stored series add a suffix (`_sats`, `_cumulative`).

mod count_with_deltas;
mod cumulative_count;
mod cumulative_fiat;
mod cumulative_value;
mod fiat;
mod supply;

pub use count_with_deltas::CountWithDeltas;
pub use cumulative_count::CumulativeCount;
pub use cumulative_fiat::CumulativeFiat;
pub use cumulative_value::CumulativeValue;
pub use fiat::Fiat;
pub use supply::Supply;

use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, PcoVecValue};

use bitview_vecs::{CachedSeries, import_cached};

fn import_stored<T: PcoVecValue>(
    db: &Database,
    name: &str,
    version: Version,
) -> Result<CachedSeries<Height, T>> {
    import_cached(db, name, version + Version::TWO)
}
