use brk_types::{Cents, Height, Timestamp};
use statedb::Reader;
use vecdb::{Budgeted, PcoVec};

/// Read-only sources needed to reconstruct block price distributions: `P` reads the prices, `T`
/// the timestamps.
pub struct ReplayInputs<'a, P, T> {
    pub history: &'a Reader<'a>,
    pub prices: &'a P,
    pub timestamps: &'a T,
}

// Only references: `Copy` whatever the readers are, which a derive would not allow.
impl<P, T> Clone for ReplayInputs<'_, P, T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<P, T> Copy for ReplayInputs<'_, P, T> {}

/// The inputs over the stored price and timestamp columns: the form a plugin's `Dependencies`
/// names, its associated type taking only a lifetime.
pub type StoredReplayInputs<'a> =
    ReplayInputs<'a, PcoVec<Height, Cents, Budgeted>, PcoVec<Height, Timestamp, Budgeted>>;
