use bitview_plugin_mappings::Vecs as MappingsVecs;
use bitview_primitives::{Count, CountSigned, PartsPerMillionSigned64};
use bitview_traversable::Traversable;
use bitview_vecs::{
    LazyPerBlockCumulativeRolling, LazyPerBlockWithDeltas, LazyWindowStartVec, shapes::Windows,
};
use brk_error::Result;
use brk_types::{Height, Version};
use vecdb::{Database, ReadableCloneableVec, Rw};

use crate::families::{CountWithDeltas, CumulativeCount};

/// A cohort's output counts; `name` gives each series name from its metric (`utxo_count` ->
/// `utxos_1d_to_1w_old_utxo_count`).
#[derive(Clone, Traversable)]
pub struct CohortOutputs<U, S> {
    /// Number of transaction outputs that are unspent at the represented block.
    pub unspent_count: U,
    /// Number of the cohort's outputs spent in each block.
    pub spent_count: S,
}

/// Counts the cohort stores itself (`utxo_count`, `spent_utxo_count_cumulative`).
pub type StoredOutputs<M = Rw> = CohortOutputs<CountWithDeltas<M>, CumulativeCount<Count, M>>;

/// Views over counts another plugin stores.
pub type OutputViews = CohortOutputs<
    LazyPerBlockWithDeltas<Count, CountSigned, PartsPerMillionSigned64>,
    LazyPerBlockCumulativeRolling<Count>,
>;

impl StoredOutputs {
    pub fn import(
        db: &Database,
        name: impl Fn(&str) -> String,
        version: Version,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Result<Self> {
        Ok(Self {
            unspent_count: CountWithDeltas::import(
                db,
                &name("utxo_count"),
                version,
                mappings,
                window_starts,
            )?,
            spent_count: CumulativeCount::import(
                db,
                &name("spent_utxo_count"),
                version + Version::ONE,
                mappings,
                window_starts,
            )?,
        })
    }

    #[inline(always)]
    pub fn push(&mut self, (unspent, spent): (Count, Count)) {
        self.unspent_count.push(unspent);
        self.spent_count.push_block(spent);
    }
}

impl OutputViews {
    /// `unspent` holds each block's count, `spent` the running total of spent outputs.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Fn(&str) -> String,
        version: Version,
        delta_version: Version,
        unspent: &impl ReadableCloneableVec<Height, Count>,
        spent: &impl ReadableCloneableVec<Height, Count>,
        mappings: &MappingsVecs,
        window_starts: &Windows<&LazyWindowStartVec>,
    ) -> Self {
        Self {
            unspent_count: LazyPerBlockWithDeltas::from_height_source(
                &name("utxo_count"),
                version,
                unspent,
                delta_version,
                mappings,
                window_starts,
            ),
            spent_count: LazyPerBlockCumulativeRolling::from_cumulative_source(
                &name("spent_utxo_count"),
                version,
                spent,
                window_starts,
                mappings,
            ),
        }
    }
}
