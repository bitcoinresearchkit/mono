use std::collections::VecDeque;

use brk_error::Result;
use brk_indexer::Indexer;
use brk_traversable::Traversable;
use brk_types::{Bitcoin, Dollars, Height, PartsPerMillion32, StoredF32, StoredU8, Version};
use derive_more::{Deref, DerefMut};
use schemars::JsonSchema;
use vecdb::{
    AnyStoredVec, AnyVec, ColumnId, Database, Exit, ReadableVec, Rw, StorageMode, VecIndex,
    VecValue, WritableVec,
};

use crate::{
    indexes,
    internal::{
        ColumnarPerBlock, LazyColumnPerBlock, NumericValue, PerBlock, PercentPerBlock,
        algo::{ExactOrderStats, FenwickTree},
        db_utils::validate_any_computed_version_or_reset,
    },
};

const VERSION: Version = Version::new(7);
const MIN_HISTORY_BLOCKS: usize = 210_000;
const WRITE_INTERVAL: usize = 10_000;
/// Above this many missing outputs, coordinate compression is faster than
/// applying exact live updates one at a time.
const BULK_BACKFILL_THRESHOLD: usize = 10_000;
const BANDS: [(f64, u8); 3] = [(0.00025, 3), (0.0005, 2), (0.001, 1)];

const THRESHOLD_COUNT: usize = 3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ExtremeThresholdId {
    Pct0_1,
    Pct0_05,
    Pct0_025,
}

const EXTREME_THRESHOLD_IDS: [ExtremeThresholdId; THRESHOLD_COUNT] = [
    ExtremeThresholdId::Pct0_1,
    ExtremeThresholdId::Pct0_05,
    ExtremeThresholdId::Pct0_025,
];

impl ExtremeThresholdId {
    fn series<T>(mut create: impl FnMut(Self) -> T) -> ThresholdVecs<T> {
        ThresholdVecs {
            threshold_pct0_1: create(Self::Pct0_1),
            threshold_pct0_05: create(Self::Pct0_05),
            threshold_pct0_025: create(Self::Pct0_025),
        }
    }
}

impl ColumnId for ExtremeThresholdId {
    type Row<T>
        = [T; THRESHOLD_COUNT]
    where
        T: VecValue;

    const VERSION: Version = Version::ONE;
    const ALL: &'static [Self] = &EXTREME_THRESHOLD_IDS;

    #[inline]
    fn index(self) -> usize {
        self as usize
    }

    #[inline]
    fn get<T: VecValue>(self, row: &Self::Row<T>) -> &T {
        &row[self.index()]
    }

    #[inline]
    fn get_mut<T: VecValue>(self, row: &mut Self::Row<T>) -> &mut T {
        &mut row[self.index()]
    }

    #[inline]
    fn from_fn<T, F>(mut create: F) -> Self::Row<T>
    where
        T: VecValue,
        F: FnMut(Self) -> T,
    {
        std::array::from_fn(|index| create(EXTREME_THRESHOLD_IDS[index]))
    }

    #[inline]
    fn map<T, U, F>(row: Self::Row<T>, create: F) -> Self::Row<U>
    where
        T: VecValue,
        U: VecValue,
        F: FnMut(T) -> U,
    {
        row.map(create)
    }
}

#[derive(Clone, Traversable)]
pub struct ThresholdVecs<T> {
    pub threshold_pct0_1: T,
    pub threshold_pct0_05: T,
    pub threshold_pct0_025: T,
}

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
/// `tail` is the current observation's top- or bottom-tail share, the three
/// `threshold` fields are the highlight boundaries, and `rank` is 0 through 3.
#[derive(Deref, DerefMut, Traversable)]
pub struct Extreme<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
{
    #[deref]
    #[deref_mut]
    #[traversable(flatten)]
    pub thresholds: ColumnarPerBlock<
        T,
        ExtremeThresholdId,
        ThresholdVecs<LazyColumnPerBlock<T, ExtremeThresholdId>>,
        M,
    >,
    pub tail: PercentPerBlock<PartsPerMillion32, M>,
    pub rank: PerBlock<StoredU8, M>,

    #[traversable(skip)]
    history: LiveHistory,
}

impl<T> Extreme<T>
where
    T: NumericValue + JsonSchema,
{
    fn forced_import(
        db: &Database,
        name: &str,
        version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let thresholds = ColumnarPerBlock::forced_import(
            db,
            &format!("{name}_thresholds"),
            version,
            |source| {
                ExtremeThresholdId::series(|threshold| {
                    let series_name = match threshold {
                        ExtremeThresholdId::Pct0_1 => format!("{name}_threshold_pct0_1"),
                        ExtremeThresholdId::Pct0_05 => format!("{name}_threshold_pct0_05"),
                        ExtremeThresholdId::Pct0_025 => format!("{name}_threshold"),
                    };
                    LazyColumnPerBlock::new(&series_name, version, source, threshold, indexes)
                })
            },
        )?;

        Ok(Self {
            thresholds,
            tail: PercentPerBlock::forced_import(db, &format!("{name}_tail"), version, indexes)?,
            rank: PerBlock::forced_import(db, &format!("{name}_rank"), version, indexes)?,
            history: LiveHistory::new(),
        })
    }

    fn compute(
        &mut self,
        indexer: &Indexer,
        source: &impl ReadableVec<Height, T>,
        config: Config,
        exit: &Exit,
    ) -> Result<()> {
        let dependency_version = source.version();
        for output in [
            &mut self.thresholds.height as &mut dyn AnyStoredVec,
            &mut self.tail.ppm.height,
            &mut self.rank.height,
        ] {
            validate_any_computed_version_or_reset(output, dependency_version)?;
        }

        let source_end = source.len();
        let start = [
            self.thresholds.height.len(),
            self.tail.ppm.height.len(),
            self.rank.height.len(),
            indexer.safe_lengths().height.to_usize(),
            source_end,
        ]
        .into_iter()
        .min()
        .unwrap_or_default();

        self.thresholds.height.any_truncate_if_needed_at(start)?;
        self.tail.ppm.height.any_truncate_if_needed_at(start)?;
        self.rank.height.any_truncate_if_needed_at(start)?;

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
            let state = if is_valid(value, config) && self.history.len() >= MIN_HISTORY_BLOCKS {
                event_state(value, &self.history, config)
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
            .filter(|&value| is_valid(value, config))
            .collect();
        sorted_values.sort_unstable_by(f64::total_cmp);
        let mut coordinates = sorted_values.clone();
        coordinates.dedup_by(|a, b| a.total_cmp(b).is_eq());

        let mut history = CoordinateHistory::new(&coordinates, config.window);
        for &value in &values[..start] {
            if is_valid(value, config) {
                history.add(value);
            }
        }

        for (height_index, &value) in values.iter().enumerate().skip(start) {
            let state = if is_valid(value, config) && history.len() >= MIN_HISTORY_BLOCKS {
                event_state(value, &history, config)
            } else {
                EventState::missing()
            };

            self.push_state(state);
            if is_valid(value, config) {
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
            .push(ExtremeThresholdId::from_fn(|threshold| match threshold {
                ExtremeThresholdId::Pct0_1 => T::from(state.thresholds.pct0_1),
                ExtremeThresholdId::Pct0_05 => T::from(state.thresholds.pct0_05),
                ExtremeThresholdId::Pct0_025 => T::from(state.thresholds.pct0_025),
            }));
        self.tail
            .ppm
            .height
            .push(PartsPerMillion32::from(state.tail));
        self.rank.height.push(StoredU8::new(state.rank));
    }

    fn write_if_needed(
        &mut self,
        height_index: usize,
        source_end: usize,
        exit: &Exit,
    ) -> Result<()> {
        if (height_index + 1).is_multiple_of(WRITE_INTERVAL) || height_index + 1 == source_end {
            let _lock = exit.lock();
            self.thresholds.write()?;
            self.tail.ppm.height.write()?;
            self.rank.height.write()?;
        }
        Ok(())
    }
}

#[derive(Traversable)]
pub struct Extremes<M: StorageMode = Rw> {
    pub coins_in_loss: Extreme<Bitcoin, M>,
    pub profit_taking: Extreme<Dollars, M>,
    pub capitulation: Extreme<Dollars, M>,
    pub peak_regret: Extreme<Dollars, M>,
    pub seller_exhaustion: Extreme<StoredF32, M>,
}

impl Extremes {
    pub(super) fn forced_import(
        db: &Database,
        parent_version: Version,
        indexes: &indexes::Vecs,
    ) -> Result<Self> {
        let version = parent_version + VERSION;
        Ok(Self {
            coins_in_loss: Extreme::forced_import(
                db,
                "rarity_meter_coins_in_loss",
                version,
                indexes,
            )?,
            profit_taking: Extreme::forced_import(
                db,
                "rarity_meter_profit_taking",
                version,
                indexes,
            )?,
            capitulation: Extreme::forced_import(
                db,
                "rarity_meter_capitulation",
                version,
                indexes,
            )?,
            peak_regret: Extreme::forced_import(db, "rarity_meter_peak_regret", version, indexes)?,
            seller_exhaustion: Extreme::forced_import(
                db,
                "rarity_meter_seller_exhaustion",
                version,
                indexes,
            )?,
        })
    }

    #[allow(clippy::too_many_arguments)]
    pub(super) fn compute(
        &mut self,
        indexer: &Indexer,
        coins_in_loss: &impl ReadableVec<Height, Bitcoin>,
        realized_profit: &impl ReadableVec<Height, Dollars>,
        realized_loss: &impl ReadableVec<Height, Dollars>,
        peak_regret: &impl ReadableVec<Height, Dollars>,
        seller_exhaustion: &impl ReadableVec<Height, StoredF32>,
        exit: &Exit,
    ) -> Result<()> {
        self.coins_in_loss
            .compute(indexer, coins_in_loss, COINS_IN_LOSS, exit)?;
        self.profit_taking
            .compute(indexer, realized_profit, REALIZED, exit)?;
        self.capitulation
            .compute(indexer, realized_loss, REALIZED, exit)?;
        self.peak_regret
            .compute(indexer, peak_regret, REALIZED, exit)?;
        self.seller_exhaustion
            .compute(indexer, seller_exhaustion, SELLER_EXHAUSTION, exit)?;
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
        let bucket = bucket(self.coordinates, value);
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
            .flat_map(|(&value, count)| std::iter::repeat_n(value, count))
            .collect();

        (
            sorted,
            HistoryWindow::Rolling {
                values: chronological,
                limit: *limit,
            },
        )
    }
}

#[derive(Clone)]
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
        historical.retain(|&value| is_valid(value, config));

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
        if !is_valid(value, config) {
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
        let mut index = [0];
        self.tree.kth(&[target], &|count: &f64| *count, &mut index);
        self.coordinates[index[0]]
    }

    fn tail(&self, value: f64, upper: bool) -> f64 {
        let bucket = bucket(self.coordinates, value);
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
}

fn event_state(value: f64, history: &impl HistoryStats, config: Config) -> EventState {
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

    EventState {
        thresholds,
        tail: history.tail(value, config.upper_tail),
        rank,
    }
}

fn is_valid(value: f64, config: Config) -> bool {
    value.is_finite() && (!config.positive_only || value > 0.0)
}

fn bucket(coordinates: &[f64], value: f64) -> usize {
    coordinates
        .binary_search_by(|candidate| candidate.total_cmp(&value))
        .expect("event value must exist in coordinate set")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn threshold_columns_match_public_fields() {
        assert_eq!(ExtremeThresholdId::ALL, EXTREME_THRESHOLD_IDS);
        let fields = ExtremeThresholdId::series(|threshold| threshold);
        assert_eq!(fields.threshold_pct0_1, ExtremeThresholdId::Pct0_1);
        assert_eq!(fields.threshold_pct0_05, ExtremeThresholdId::Pct0_05);
        assert_eq!(fields.threshold_pct0_025, ExtremeThresholdId::Pct0_025);
    }

    fn history(values: &[f64], config: Config) -> LiveHistory {
        let mut history = LiveHistory::new();
        history.rebuild(values.to_vec(), config);
        history
    }

    fn assert_same_state(left: EventState, right: EventState) {
        assert_eq!(left.thresholds.pct0_1, right.thresholds.pct0_1);
        assert_eq!(left.thresholds.pct0_05, right.thresholds.pct0_05);
        assert_eq!(left.thresholds.pct0_025, right.thresholds.pct0_025);
        assert_eq!(left.tail, right.tail);
        assert_eq!(left.rank, right.rank);
    }

    #[test]
    fn upper_tail_includes_current_observation() {
        let history = history(&(1..=100).map(f64::from).collect::<Vec<_>>(), REALIZED);

        let state = event_state(101.0, &history, REALIZED);
        assert!((state.tail - 1.0 / 101.0).abs() < f64::EPSILON);
        assert_eq!(state.rank, 3);
        assert_eq!(state.thresholds.pct0_025, 99.0);
        assert!(state.thresholds.pct0_1 <= state.thresholds.pct0_05);
        assert!(state.thresholds.pct0_05 <= state.thresholds.pct0_025);
    }

    #[test]
    fn lower_tail_includes_current_observation() {
        let history = history(
            &(1..=100).map(f64::from).collect::<Vec<_>>(),
            SELLER_EXHAUSTION,
        );
        let state = event_state(0.0, &history, SELLER_EXHAUSTION);

        assert!((state.tail - 1.0 / 101.0).abs() < f64::EPSILON);
        assert_eq!(state.rank, 3);
        assert_eq!(state.thresholds.pct0_025, 1.0);
        assert!(state.thresholds.pct0_1 >= state.thresholds.pct0_05);
        assert!(state.thresholds.pct0_05 >= state.thresholds.pct0_025);
    }

    #[test]
    fn incremental_history_matches_coordinate_history() {
        let values = [3.0, 1.0, 2.0, 2.0, -1.0, 5.0, 4.0, 5.0];
        let mut coordinates = values.to_vec();
        coordinates.sort_unstable_by(f64::total_cmp);
        coordinates.dedup_by(|a, b| a.total_cmp(b).is_eq());

        let mut coordinate = CoordinateHistory::new(&coordinates, WindowKind::Full);
        let mut incremental = LiveHistory::new();

        for value in values {
            if coordinate.len() > 0 {
                assert_same_state(
                    event_state(value, &coordinate, REALIZED),
                    event_state(value, &incremental, REALIZED),
                );
            }
            coordinate.add(value);
            incremental.observe(value, REALIZED);
        }
    }

    #[test]
    fn rolling_history_evicts_the_oldest_observation() {
        let values: Vec<_> = (1..=MIN_HISTORY_BLOCKS + 1)
            .map(|value| value as f64)
            .collect();
        let mut history = history(&values, SELLER_EXHAUSTION);

        assert_eq!(history.len(), MIN_HISTORY_BLOCKS);
        assert_eq!(history.stats.kth(0), 2.0);

        history.observe((MIN_HISTORY_BLOCKS + 2) as f64, SELLER_EXHAUSTION);

        assert_eq!(history.len(), MIN_HISTORY_BLOCKS);
        assert_eq!(history.stats.kth(0), 3.0);
    }

    #[test]
    fn rollback_rebuild_replaces_incremental_state() {
        let mut history = history(&[1.0, 2.0, 3.0], REALIZED);
        history.observe(4.0, REALIZED);
        assert!(history.is_current(4, WindowKind::Full));

        history.rebuild(vec![1.0, 2.0], REALIZED);

        assert!(history.is_current(2, WindowKind::Full));
        assert_eq!(history.len(), 2);
        assert_eq!(history.stats.kth(0), 1.0);
        assert_eq!(history.stats.kth(1), 2.0);
    }
}
