use bitview_traversable::Traversable;
use vecdb::{Rw, StorageMode};

use super::{CohortVecs, Sources};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// Uses all UTXO age ranges.
    pub all: CohortVecs,
    /// Uses short-term-holder UTXO age ranges younger than 150 days.
    pub sth: CohortVecs,
    /// Uses long-term-holder UTXO age ranges at least 150 days old.
    pub lth: CohortVecs,
    /// Height-indexed source matrices for all, short-term-holder, and
    /// long-term-holder cointime aggregates.
    pub sources: Sources<M>,
}
