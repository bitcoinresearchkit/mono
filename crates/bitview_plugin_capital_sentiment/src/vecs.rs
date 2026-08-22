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

    /// Whether the stateful Capital Sentiment strategy holds bitcoin rather than
    /// its short position on the indexed day. Using each day's final available
    /// block, it enters when spot crosses from below to at or above the
    /// short-term-holder (STH) capitalized price and exits in `limbo`,
    /// `deep_bear`, `bear`, or `early_bear`; otherwise it keeps the prior day's
    /// position. It starts short, and a missing phase does not itself cause an
    /// exit.
    pub is_long: DailyMetric<StoredBool, M>,
    /// Whether the stateful Capital Sentiment strategy holds its short position
    /// rather than bitcoin on the indexed day; exactly the complement of the
    /// long flag.
    pub is_short: LazyDailyMetric<StoredBool, StoredBool>,
    /// Daily Capital Sentiment phase describing whether spot is above or below
    /// the value-weighted mean acquisition prices of all, short-term-holder
    /// (STH), and long-term-holder (LTH) unspent supply. Being above more of
    /// these capitalized-price benchmarks indicates a stronger market
    /// structure; their ordering distinguishes early, established, and weakening
    /// phases. The one-year spot-price simple moving average is used only for
    /// confirmation and disambiguation. Classification uses the final available
    /// block of the day. Let `P`, `A`, `S`, `L`, and `M` be spot, all-chain,
    /// STH, and LTH capitalized price, and the one-year average. Comparisons
    /// count equality as above. The core bull label is `raging_bull` when
    /// `A > M`, otherwise `bull`; the core bear label is `deep_bear` when
    /// `L > A`, otherwise `bear`. With `M` available, the ordered rules are:
    /// below all four references gives the core bear label; below `S` with
    /// exactly two references above spot gives `limbo`; when `S > L`, being
    /// below either `A` or `L`, or below `M`, gives `early_bear`; at or above
    /// both `S` and `M` gives the core bull label when also at or above `A` and
    /// `L`, otherwise `early_bull`; below `S` but at or above `A`, `L`, and `M`
    /// gives `weak_bull`; otherwise being at or above `S` gives
    /// `cautious_bull`, being at or above `M` gives `hopeful_bull`, and the
    /// remainder is `early_bear`. When `M` is unavailable, `S > L` uses the
    /// same capitalized-price structure alone: at or above `S` is core bull if
    /// also above both slow references (`A` and `L`), otherwise `early_bull`;
    /// below `S` is `weak_bull` above both slow references, `early_bear` above
    /// either one, and core bear below both. When `S <= L`, below `S` is core
    /// bear; at or above `S` is core bull above both slow references,
    /// `early_bull` above either one, and `cautious_bull` below both.
    /// Values are `raging_bull`, `bull`, `cautious_bull`, `hopeful_bull`,
    /// `early_bull`, `weak_bull`, `limbo`, `deep_bear`, `bear`, or
    /// `early_bear`. Missing when spot or any capitalized-price input is absent,
    /// nonpositive, or non-finite; the moving average itself may be unavailable.
    pub phase: LazyDailyMetric<Option<CapitalSentimentPhase>, StoredU8>,
    /// Coarse Capital Sentiment direction, where positive values are bullish and
    /// negative values bearish. It is derived from the daily phase:
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
