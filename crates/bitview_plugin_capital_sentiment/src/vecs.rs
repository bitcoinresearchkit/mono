mod compute;
mod import;

use bitview_plugin::{Plugin, PluginGate, PluginStorage};
use bitview_traversable::Traversable;
use brk_types::{CapitalSentimentPhase, StoredBool, StoredI8, StoredU8};
use vecdb::{Database, Rw, StorageMode};

use crate::STORAGE;
use bitview_compute::{DailyMetric, LazyDailyMetric};

#[derive(Traversable)]
pub struct Vecs<M: StorageMode = Rw> {
    #[traversable(skip)]
    plugin_gate: PluginGate,
    #[traversable(skip)]
    db: Database,

    /// Compact daily source of truth.
    #[traversable(hidden)]
    phase_code: DailyMetric<StoredU8, M>,

    /// Whether the stateful capital-sentiment strategy is long for the indexed
    /// day. It enters when spot crosses from below to at or above the STH
    /// capitalized price and exits when the phase is classified as sell.
    pub is_long: DailyMetric<StoredBool, M>,
    /// Whether the stateful capital-sentiment strategy is short for the indexed
    /// day; exactly the complement of `is_long`.
    pub is_short: LazyDailyMetric<StoredBool, StoredBool>,
    /// Daily investor phase classified from the final available block's spot
    /// price relative to the all, STH, and LTH capitalized prices, with the
    /// one-year spot-price SMA used only for confirmation and disambiguation.
    /// Values are `raging_bull`, `bull`, `cautious_bull`, `hopeful_bull`,
    /// `early_bull`, `weak_bull`, `limbo`, `deep_bear`, `bear`, or
    /// `early_bear`. Missing when spot or any capitalized-price input is absent,
    /// nonpositive, or non-finite; the SMA itself may be unavailable.
    pub phase: LazyDailyMetric<Option<CapitalSentimentPhase>, StoredU8>,
    /// Coarse directional score derived from `capital_sentiment_phase`:
    /// `raging_bull`, `bull`, and `early_bull` map to 2; `cautious_bull`,
    /// `hopeful_bull`, and `weak_bull` map to 1; `limbo` maps to -1; and
    /// `deep_bear`, `bear`, and `early_bear` map to -2. Missing when the phase
    /// is missing.
    pub score: LazyDailyMetric<Option<StoredI8>, Option<CapitalSentimentPhase>>,
}

impl<M: StorageMode> Plugin for Vecs<M>
where
    Self: Traversable + Send + Sync,
{
    fn storage(&self) -> PluginStorage {
        STORAGE
    }

    fn gate(&self) -> &PluginGate {
        &self.plugin_gate
    }
}
