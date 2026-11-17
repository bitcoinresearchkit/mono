use std::collections::VecDeque;

use bitview_compute::{ExactOrderStats, NumericValue, prepare_computed, write_chunks, write_locked};
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

/// Which observations an extreme's history keeps: all prior ones or only the latest `window`,
/// and any finite value or only positive ones.
#[derive(Clone, Copy)]
pub(super) struct Config {
    window: Option<usize>,
    positive_only: bool,
}

impl Config {
    fn accepts(self, value: f64) -> bool {
        value.is_finite() && (!self.positive_only || value > 0.0)
    }
}

pub(super) const REALIZED: Config = Config {
    window: None,
    positive_only: false,
};

pub(super) const SUPPLY_IN_LOSS: Config = Config {
    window: None,
    positive_only: true,
};

pub(super) const SELL_SIDE_RISK: Config = Config {
    window: Some(MIN_HISTORY_BLOCKS),
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

    history: M::WriteOnly<History>,
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
            history: History::default(),
        })
    }

    /// Each missing block's bands, then the block joins the history. The history carries on when
    /// it stands at the first missing block, and is read again from the source otherwise.
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

        let values = |from| -> Vec<f64> {
            source
                .collect_range_at(from, source_end)
                .into_iter()
                .map(Into::into)
                .collect()
        };
        let pending = if self.history.processed == start {
            values(start)
        } else {
            let mut prior = values(0);
            let pending = prior.split_off(start);
            self.history.rebuild(prior, config);
            pending
        };

        for chunk in write_chunks(start..source_end) {
            for &value in &pending[chunk.start - start..chunk.end - start] {
                let bands = self.history.bands();
                for (vec, band) in self.bands.iter_mut().zip(bands) {
                    vec.height.push(T::from(band));
                }
                self.history.observe(value, config);
            }
            write_locked(self.iter_any_stored_mut(), exit)?;
        }
        Ok(())
    }
}

/// The accepted observations before a block, in order of value.
#[derive(Default)]
struct History {
    stats: ExactOrderStats,
    /// Source values taken in, accepted or not: the height the history stands at.
    processed: usize,
    /// The accepted values in order of arrival, kept only under a window.
    recent: VecDeque<f64>,
}

impl History {
    fn rebuild(&mut self, mut prior: Vec<f64>, config: Config) {
        self.processed = prior.len();
        prior.retain(|&value| config.accepts(value));
        self.recent.clear();
        if let Some(window) = config.window {
            prior.drain(..prior.len().saturating_sub(window));
            self.recent.extend(&prior);
        }
        self.stats = ExactOrderStats::from_unsorted(prior);
    }

    fn observe(&mut self, value: f64, config: Config) {
        self.processed += 1;
        if !config.accepts(value) {
            return;
        }

        self.stats.insert(value);
        if let Some(window) = config.window {
            self.recent.push_back(value);
            if self.recent.len() > window {
                // A value expires from the window that took it in.
                let expired = self.recent.pop_front().unwrap();
                let removed = self.stats.remove(expired);
                debug_assert!(removed, "expired value not in the history");
            }
        }
    }

    /// The rarity percentiles of the history once it is long enough, else missing. They describe
    /// the history alone, whether or not the next observation joins it.
    fn bands(&self) -> [f64; RARITY_PERCENTILES_LEN] {
        if self.stats.count() >= MIN_HISTORY_BLOCKS {
            self.stats.percentiles(RARITY_PERCENTILES)
        } else {
            [f64::NAN; RARITY_PERCENTILES_LEN]
        }
    }
}
