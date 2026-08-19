use bitview_traversable::Traversable;
use brk_types::{PartsPerMillionSigned64, StoredF64};
use vecdb::{Rw, StorageMode};

use bitview_compute::{PerBlock, PercentPerBlock};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    /// Supply inflation rate multiplied by the activity-to-vaultedness ratio.
    pub inflation_rate: PercentPerBlock<PartsPerMillionSigned64, M>,
    /// Native transaction velocity multiplied by the
    /// activity-to-vaultedness ratio.
    pub tx_velocity_native: PerBlock<StoredF64, M>,
    /// Fiat transaction velocity multiplied by the
    /// activity-to-vaultedness ratio.
    pub tx_velocity_fiat: PerBlock<StoredF64, M>,
}
