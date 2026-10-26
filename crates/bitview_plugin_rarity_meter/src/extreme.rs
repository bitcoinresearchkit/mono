use std::{collections::VecDeque, iter::repeat_n};

use bitview_compute::{
    ExactOrderStats, FenwickTree, NumericValue, WRITE_INTERVAL, nearest_rank, prepare_computed,
    write_locked,
};
use bitview_primitives::{RARITY_PERCENTILES, RARITY_PERCENTILES_LEN};
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, LazyPerBlock, PerBlock, shapes::RarityPercentiles};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Height, Version};
use schemars::JsonSchema;
use vecdb::{
    AnyVec, Database, Ident, ReadableCloneableVec, ReadableVec, Rw, StorageMode, VecIndex,
    WritableVec,
};

const MIN_HISTORY_BLOCKS: usize = 210_000;
/// Above this many missing outputs, coordinate compression is faster than
/// applying exact live updates one at a time.
const BULK_BACKFILL_THRESHOLD: usize = 10_000;

#[derive(Clone, Copy, PartialEq, Eq)]
enum WindowKind {
    Full,
    Rolling(usize),
}

/// Which observations an extreme's history keeps: all prior ones or a rolling window, and any
/// finite value or only positive ones.
#[derive(Clone, Copy)]
pub(super) struct Config {
    window: WindowKind,
    positive_only: bool,
}

impl Config {
    fn accepts(self, value: f64) -> bool {
        value.is_finite() && (!self.positive_only || value > 0.0)
    }
}

pub(super) const REALIZED: Config = Config {
    window: WindowKind::Full,
    positive_only: false,
};

pub(super) const SUPPLY_IN_LOSS: Config = Config {
    window: WindowKind::Full,
    positive_only: true,
};

pub(super) const SELL_SIDE_RISK: Config = Config {
    window: WindowKind::Rolling(MIN_HISTORY_BLOCKS),
    positive_only: true,
};

/// One metric and its historical percentile bands: the source values at each rarity percentile
/// of its prior observations, so the metric reads against its own history as a price against its
/// bands.
#[derive(Traversable)]
pub struct Extreme<T, M: StorageMode = Rw>
where
    T: NumericValue + JsonSchema,
{
    /// The metric itself, under its own id.
    pub value: LazyPerBlock<T>,
    /// The source value at this percentile of the accepted observations before the represented
    /// one (the series' model sets the history).
    #[traversable(flatten)]
    #[traversable(lead = "Its rarity bands: the values as rare as each percentile of its history.")]
    pub bands: RarityPercentiles<PerBlock<T, M>>,

    history: M::WriteOnly<LiveHistory>,
}

impl<T> Extreme<T>
where
    T: NumericValue + JsonSchema,
{
    /// `source` is the metric; its id with `_rarity` prefixes the bands' (`supply_in_loss_rarity_p1`).
    pub fn import(
        db: &Database,
        source: &impl ReadableCloneableVec<Height, T>,
        version: Version,
        mappings: &IndexSources,
    ) -> Result<Self> {
        let name = format!("{}_rarity", source.name());
        Ok(Self {
            value: LazyPerBlock::from_height_source::<Ident>(
                source.name(),
                version,
                source,
                mappings,
            ),
            bands: RarityPercentiles::try_from_fn(|id| {
                PerBlock::import(db, &format!("{name}_{}", id.suffix()), version, mappings)
            })?,
            history: LiveHistory::new(),
        })
    }

    pub(super) fn compute(
        &mut self,
        starting_height: Height,
        config: Config,
        exit: &Exit,
    ) -> Result<()> {
        let source = &self.value.height.clone();
        let source_end = source.len();
        let start = prepare_computed(
            self.iter_any_stored_mut().collect::<Vec<_>>(),
            source.version(),
            starting_height.to_usize().min(source_end),
            exit,
        )?;

        if source_end - start > BULK_BACKFILL_THRESHOLD {
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
            self.push_bands(bands(&self.history));
            self.history.observe(value, config);
            self.write_if_needed(height_index, source_end, exit)?;
        }

        Ok(())
    }

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
            self.push_bands(bands(&history));
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

    fn push_bands(&mut self, bands: [f64; RARITY_PERCENTILES_LEN]) {
        for (vec, band) in self.bands.iter_mut().zip(bands) {
            vec.height.push(T::from(band));
        }
    }

    fn write_if_needed(
        &mut self,
        height_index: usize,
        source_end: usize,
        exit: &Exit,
    ) -> Result<()> {
        if (height_index + 1).is_multiple_of(WRITE_INTERVAL) || height_index + 1 == source_end {
            write_locked(self.iter_any_stored_mut(), exit)?;
        }
        Ok(())
    }
}

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
            tree: FenwickTree::new(coordinates.len()),
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
            // A value expires from the window that took it in.
            let removed = self.stats.remove(expired);
            debug_assert!(removed, "expired value not in the history");
        }
    }
}

trait HistoryStats {
    fn len(&self) -> usize;
    /// Percentiles (ascending) at the nearest rank, in one lookup.
    fn percentiles<const N: usize>(&self, ps: [f64; N]) -> [f64; N];
}

impl HistoryStats for CoordinateHistory<'_> {
    fn len(&self) -> usize {
        self.len
    }

    fn percentiles<const N: usize>(&self, ps: [f64; N]) -> [f64; N] {
        let targets = ps.map(|p| nearest_rank(self.len, p) as f64);
        self.tree
            .kth(targets, &|count: &f64| *count)
            .map(|bucket| self.coordinates[bucket])
    }
}

impl HistoryStats for LiveHistory {
    fn len(&self) -> usize {
        self.stats.count()
    }

    fn percentiles<const N: usize>(&self, ps: [f64; N]) -> [f64; N] {
        self.stats.percentiles(ps)
    }
}

/// The bands before an observation: its history's rarity percentiles once that history is long
/// enough, else missing. They describe the history alone, whether or not the observation joins it.
fn bands(history: &impl HistoryStats) -> [f64; RARITY_PERCENTILES_LEN] {
    if history.len() >= MIN_HISTORY_BLOCKS {
        history.percentiles(RARITY_PERCENTILES)
    } else {
        [f64::NAN; RARITY_PERCENTILES_LEN]
    }
}
