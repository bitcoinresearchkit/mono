use std::{collections::VecDeque, iter::repeat_n};

use bitview_compute::{ExactOrderStats, FenwickTree, NumericValue, prepare_computed};
use bitview_plugin_indexer::Indexer;
use bitview_primitives::{PartsPerMillion32, Rank};
use bitview_traversable::Traversable;
use bitview_vecs::{FixedRatioPerBlock, IndexSources, PerBlock};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{AnyStoredVec, Database, ReadableVec, Rw, StorageMode, VecIndex, WritableVec};

use crate::threshold_vecs::ThresholdVecs;

const MIN_HISTORY_BLOCKS: usize = 210_000;
const WRITE_INTERVAL: usize = 10_000;
/// Above this many missing outputs, coordinate compression is faster than
/// applying exact live updates one at a time.
const BULK_BACKFILL_THRESHOLD: usize = 10_000;
const BANDS: [(f64, u8); 3] = [(0.00025, 3), (0.0005, 2), (0.001, 1)];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum WindowKind {
    Full,
    Rolling(usize),
}

#[derive(Clone, Copy)]
struct Config {
    upper_tail: bool,
    window: WindowKind,
    positive_only: bool,
}

impl Config {
    fn accepts(self, value: f64) -> bool {
        value.is_finite() && (!self.positive_only || value > 0.0)
    }
}

const REALIZED: Config = Config {
    upper_tail: true,
    window: WindowKind::Full,
    positive_only: false,
};

const COINS_IN_LOSS: Config = Config {
    upper_tail: true,
    window: WindowKind::Full,
    positive_only: true,
};

const SELLER_EXHAUSTION: Config = Config {
    upper_tail: false,
    window: WindowKind::Rolling(MIN_HISTORY_BLOCKS),
    positive_only: true,
};

/// Historical extremeness of one metric.
///
/// `tail` is the represented observation's top- or bottom-tail share, the three
/// `threshold` fields are the highlight boundaries, and `rank` is 0 through 3.
#[derive(Deref, DerefMut, Traversable)]
pub struct Extreme<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    /// Source values corresponding to rare historical tail probabilities before
    /// the represented observation. An upper-tail event is extreme at or above
    /// its boundary; a lower-tail event is extreme at or below it. The series'
    /// model determines the history and direction.
    pub thresholds: ThresholdVecs<PerBlock<T, M>>,
    /// Historical tail share: the fraction of accepted observations at least as
    /// extreme as the represented source value in the configured upper or lower
    /// tail. A smaller percentage means a rarer observation. The represented
    /// observation is included in both numerator and denominator.
    pub tail: FixedRatioPerBlock<PartsPerMillion32, M>,
    /// Discrete extremeness rank, where a higher value means a rarer event: 3 at
    /// or beyond the 0.025% tail boundary, 2 at or beyond 0.05%, 1 at or beyond
    /// 0.1%, and 0 otherwise or while unavailable. The series' model determines
    /// whether boundaries are crossed upward or downward.
    pub rank: PerBlock<Rank, M>,

    history: M::WriteOnly<LiveHistory>,
}

impl<T> Extreme<T>
where
    T: NumericValue + JsonSchema,
{
    pub fn import(
        db: &Database,
        name: &str,
        version: Version,
        mappings: &IndexSources,
    ) -> Result<Self> {
        let version = version + Version::ONE;
        let thresholds = ThresholdVecs {
            threshold_pct0_1: PerBlock::import(
                db,
                &format!("{name}_threshold_pct0_1"),
                version,
                mappings,
            )?,
            threshold_pct0_05: PerBlock::import(
                db,
                &format!("{name}_threshold_pct0_05"),
                version,
                mappings,
            )?,
            threshold_pct0_025: PerBlock::import(
                db,
                &format!("{name}_threshold"),
                version,
                mappings,
            )?,
        };

        Ok(Self {
            thresholds,
            tail: FixedRatioPerBlock::import(db, &format!("{name}_tail"), version, mappings)?,
            rank: PerBlock::import(db, &format!("{name}_rank"), version, mappings)?,
            history: LiveHistory::new(),
        })
    }

    pub fn compute_coins_in_loss(
        &mut self,
        indexer: &Indexer,
        source: &impl ReadableVec<Height, T>,
        exit: &Exit,
    ) -> Result<()> {
        self.compute(indexer.safe_lengths().height, source, COINS_IN_LOSS, exit)
    }

    pub fn compute_realized(
        &mut self,
        indexer: &Indexer,
        source: &impl ReadableVec<Height, T>,
        exit: &Exit,
    ) -> Result<()> {
        self.compute(indexer.safe_lengths().height, source, REALIZED, exit)
    }

    pub fn compute_seller_exhaustion(
        &mut self,
        indexer: &Indexer,
        source: &impl ReadableVec<Height, T>,
        exit: &Exit,
    ) -> Result<()> {
        self.compute(
            indexer.safe_lengths().height,
            source,
            SELLER_EXHAUSTION,
            exit,
        )
    }

    fn compute(
        &mut self,
        starting_height: Height,
        source: &impl ReadableVec<Height, T>,
        config: Config,
        exit: &Exit,
    ) -> Result<()> {
        let source_end = source.len();
        let start = prepare_computed(
            [
                &mut self.thresholds.threshold_pct0_1.height as &mut dyn AnyStoredVec,
                &mut self.thresholds.threshold_pct0_05.height,
                &mut self.thresholds.threshold_pct0_025.height,
                &mut self.tail.ppm.height,
                &mut self.rank.height,
            ],
            source.version(),
            starting_height.to_usize().min(source_end),
            exit,
        )?;
        self.write(exit)?;

        if source_end.saturating_sub(start) > BULK_BACKFILL_THRESHOLD {
            self.compute_bulk(source, start, source_end, config, exit)
        } else {
            self.compute_incremental(source, start, source_end, config, exit)
        }
    }

    fn compute_incremental(
        &mut self,
        source: &impl ReadableVec<Height, T>,
        start: usize,
        source_end: usize,
        config: Config,
        exit: &Exit,
    ) -> Result<()> {
        let pending = if self.history.is_current(start, config.window) {
            source
                .collect_range_at(start, source_end)
                .into_iter()
                .map(Into::into)
                .collect()
        } else {
            let mut values: Vec<f64> = source
                .collect_range_at(0, source_end)
                .into_iter()
                .map(Into::into)
                .collect();
            let pending = values.split_off(start);
            self.history.rebuild(values, config);
            pending
        };

        for (offset, value) in pending.into_iter().enumerate() {
            let height_index = start + offset;
            let state = if config.accepts(value) && self.history.len() >= MIN_HISTORY_BLOCKS {
                EventState::new(value, &self.history, config)
            } else {
                EventState::missing()
            };

            self.push_state(state);
            self.history.observe(value, config);
            self.write_if_needed(height_index, source_end, exit)?;
        }

        Ok(())
    }

    /// Preserve the coordinate-compressed Fenwick path for large backfills.
    fn compute_bulk(
        &mut self,
        source: &impl ReadableVec<Height, T>,
        start: usize,
        source_end: usize,
        config: Config,
        exit: &Exit,
    ) -> Result<()> {
        let values: Vec<f64> = source
            .collect_range_at(0, source_end)
            .into_iter()
            .map(Into::into)
            .collect();
        let mut sorted_values: Vec<f64> = values
            .iter()
            .copied()
            .filter(|&value| config.accepts(value))
            .collect();
        sorted_values.sort_unstable_by(f64::total_cmp);
        let mut coordinates = sorted_values.clone();
        coordinates.dedup_by(|a, b| a.total_cmp(b).is_eq());

        let mut history = CoordinateHistory::new(&coordinates, config.window);
        for &value in &values[..start] {
            if config.accepts(value) {
                history.add(value);
            }
        }

        for (height_index, &value) in values.iter().enumerate().skip(start) {
            let state = if config.accepts(value) && history.len() >= MIN_HISTORY_BLOCKS {
                EventState::new(value, &history, config)
            } else {
                EventState::missing()
            };

            self.push_state(state);
            if config.accepts(value) {
                history.add(value);
            }
            self.write_if_needed(height_index, source_end, exit)?;
        }

        let (sorted_values, window) = match config.window {
            WindowKind::Full => (sorted_values, HistoryWindow::Full),
            WindowKind::Rolling(_) => history.rolling_snapshot(),
        };
        self.history
            .replace_from_sorted(source_end, sorted_values, window);
        Ok(())
    }

    fn push_state(&mut self, state: EventState) {
        self.thresholds
            .threshold_pct0_1
            .height
            .push(T::from(state.thresholds.pct0_1));
        self.thresholds
            .threshold_pct0_05
            .height
            .push(T::from(state.thresholds.pct0_05));
        self.thresholds
            .threshold_pct0_025
            .height
            .push(T::from(state.thresholds.pct0_025));
        self.tail
            .ppm
            .height
            .push(PartsPerMillion32::from(state.tail));
        self.rank.height.push(Rank::new(state.rank));
    }

    fn write_if_needed(
        &mut self,
        height_index: usize,
        source_end: usize,
        exit: &Exit,
    ) -> Result<()> {
        if (height_index + 1).is_multiple_of(WRITE_INTERVAL) || height_index + 1 == source_end {
            self.write(exit)?;
        }
        Ok(())
    }

    fn write(&mut self, exit: &Exit) -> Result<()> {
        let _lock = exit.lock();
        for v in self.thresholds.iter_mut() {
            v.height.write()?;
        }
        self.tail.ppm.height.write()?;
        self.rank.height.write()?;
        Ok(())
    }
}

#[derive(Clone)]
enum HistoryWindow<T> {
    Full,
    Rolling { values: VecDeque<T>, limit: usize },
}

impl<T> HistoryWindow<T> {
    fn new(kind: WindowKind) -> Self {
        match kind {
            WindowKind::Full => Self::Full,
            WindowKind::Rolling(limit) => Self::Rolling {
                values: VecDeque::with_capacity(limit),
                limit,
            },
        }
    }

    fn kind(&self) -> WindowKind {
        match self {
            Self::Full => WindowKind::Full,
            Self::Rolling { limit, .. } => WindowKind::Rolling(*limit),
        }
    }

    fn push(&mut self, value: T) -> Option<T> {
        let Self::Rolling { values, limit } = self else {
            return None;
        };
        values.push_back(value);
        (values.len() > *limit).then(|| values.pop_front().unwrap())
    }
}

struct CoordinateHistory<'a> {
    coordinates: &'a [f64],
    tree: FenwickTree<f64>,
    len: usize,
    window: HistoryWindow<usize>,
}

impl<'a> CoordinateHistory<'a> {
    fn new(coordinates: &'a [f64], window: WindowKind) -> Self {
        Self {
            coordinates,
            tree: FenwickTree::new(coordinates.len().max(1)),
            len: 0,
            window: HistoryWindow::new(window),
        }
    }

    fn add(&mut self, value: f64) {
        let bucket = self.bucket(value);
        self.tree.add(bucket, &1.0);
        self.len += 1;

        if let Some(expired) = self.window.push(bucket) {
            self.tree.add(expired, &-1.0);
            self.len -= 1;
        }
    }

    fn rolling_snapshot(&self) -> (Vec<f64>, HistoryWindow<f64>) {
        let HistoryWindow::Rolling { values, limit } = &self.window else {
            unreachable!("rolling snapshot requested for full history")
        };

        let mut counts = vec![0_usize; self.coordinates.len()];
        let chronological: VecDeque<_> = values
            .iter()
            .map(|&bucket| {
                counts[bucket] += 1;
                self.coordinates[bucket]
            })
            .collect();
        let sorted = self
            .coordinates
            .iter()
            .zip(counts)
            .flat_map(|(&value, count)| repeat_n(value, count))
            .collect();

        (
            sorted,
            HistoryWindow::Rolling {
                values: chronological,
                limit: *limit,
            },
        )
    }

    fn bucket(&self, value: f64) -> usize {
        self.coordinates
            .binary_search_by(|candidate| candidate.total_cmp(&value))
            .expect("event value must exist in coordinate set")
    }
}

struct LiveHistory {
    stats: ExactOrderStats,
    processed: usize,
    window: HistoryWindow<f64>,
}

impl LiveHistory {
    fn new() -> Self {
        Self {
            stats: ExactOrderStats::new(0),
            processed: 0,
            window: HistoryWindow::Full,
        }
    }

    fn is_current(&self, processed: usize, window: WindowKind) -> bool {
        self.processed == processed && self.window.kind() == window
    }

    fn rebuild(&mut self, mut historical: Vec<f64>, config: Config) {
        self.processed = historical.len();
        historical.retain(|&value| config.accepts(value));

        self.window = HistoryWindow::new(config.window);
        if let HistoryWindow::Rolling { values, limit } = &mut self.window {
            let keep_from = historical.len().saturating_sub(*limit);
            historical.drain(..keep_from);
            values.extend(historical.iter().copied());
        }

        self.stats = ExactOrderStats::from_unsorted(historical);
    }

    fn replace_from_sorted(
        &mut self,
        processed: usize,
        sorted_values: Vec<f64>,
        window: HistoryWindow<f64>,
    ) {
        if let HistoryWindow::Rolling { values, .. } = &window {
            debug_assert_eq!(sorted_values.len(), values.len());
        }
        self.processed = processed;
        self.stats = ExactOrderStats::from_sorted(sorted_values);
        self.window = window;
    }

    fn observe(&mut self, value: f64, config: Config) {
        self.processed += 1;
        if !config.accepts(value) {
            return;
        }

        self.stats.insert(value);
        if let Some(expired) = self.window.push(value) {
            assert!(self.stats.remove(expired));
        }
    }
}

trait HistoryStats {
    fn len(&self) -> usize;
    fn quantile(&self, percentile: f64) -> f64;
    fn tail(&self, value: f64, upper: bool) -> f64;
}

impl HistoryStats for CoordinateHistory<'_> {
    fn len(&self) -> usize {
        self.len
    }

    fn quantile(&self, percentile: f64) -> f64 {
        let target = ((self.len - 1) as f64 * percentile).floor();
        let index = self.tree.kth([target], &|count: &f64| *count);
        self.coordinates[index[0]]
    }

    fn tail(&self, value: f64, upper: bool) -> f64 {
        let bucket = self.bucket(value);
        let less = if bucket == 0 {
            0.0
        } else {
            self.tree.prefix_sum(bucket - 1)
        };
        let less_or_equal = self.tree.prefix_sum(bucket);
        let count = if upper {
            self.len as f64 - less
        } else {
            less_or_equal
        };
        (count + 1.0) / (self.len as f64 + 1.0)
    }
}

impl HistoryStats for LiveHistory {
    fn len(&self) -> usize {
        self.stats.len()
    }

    fn quantile(&self, percentile: f64) -> f64 {
        let index = ((self.stats.len() - 1) as f64 * percentile).floor() as usize;
        self.stats.kth(index)
    }

    fn tail(&self, value: f64, upper: bool) -> f64 {
        let count = if upper {
            self.stats.len() - self.stats.count_lt(value)
        } else {
            self.stats.count_le(value)
        };
        (count as f64 + 1.0) / (self.stats.len() as f64 + 1.0)
    }
}

struct EventState {
    thresholds: EventThresholds,
    tail: f64,
    rank: u8,
}

struct EventThresholds {
    pct0_1: f64,
    pct0_05: f64,
    pct0_025: f64,
}

impl EventState {
    fn missing() -> Self {
        Self {
            thresholds: EventThresholds {
                pct0_1: f64::NAN,
                pct0_05: f64::NAN,
                pct0_025: f64::NAN,
            },
            tail: f64::NAN,
            rank: 0,
        }
    }

    fn new(value: f64, history: &impl HistoryStats, config: Config) -> Self {
        let percentile = |tail: f64| {
            if config.upper_tail { 1.0 - tail } else { tail }
        };
        let boundary = |tail: f64| history.quantile(percentile(tail));
        let thresholds = EventThresholds {
            pct0_1: boundary(BANDS[2].0),
            pct0_05: boundary(BANDS[1].0),
            pct0_025: boundary(BANDS[0].0),
        };
        let rank = [
            (thresholds.pct0_025, BANDS[0].1),
            (thresholds.pct0_05, BANDS[1].1),
            (thresholds.pct0_1, BANDS[2].1),
        ]
        .into_iter()
        .find_map(|(boundary, rank)| {
            let reached = if config.upper_tail {
                value >= boundary
            } else {
                value <= boundary
            };
            reached.then_some(rank)
        })
        .unwrap_or_default();

        Self {
            thresholds,
            tail: history.tail(value, config.upper_tail),
            rank,
        }
    }
}
