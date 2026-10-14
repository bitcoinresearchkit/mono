use std::ops::Range;

use bitview_compute::{prepare_computed, write_chunks, write_locked};
use bitview_primitives::{RARITY_PERCENTILES_LEN, RarityPercentileId, Score};
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, PerBlock, Price, shapes::RarityPercentiles};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableVec, Rw, StorageMode, WritableVec};

use super::{Component, START_HEIGHT, component};

#[derive(Traversable)]
pub struct RarityMeterInner<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// Historical price boundaries that any of the meter's reference models calls rare.
    /// Lower-percentile bands represent unusually low spot valuations;
    /// upper-percentile bands represent unusually high valuations. Lower boundaries from
    /// 0.1% through 5% use the highest corresponding component boundary and upper
    /// boundaries from 95% through 99.9% the lowest, so spot crosses one as soon as it
    /// crosses that boundary for any model. The 10th through 90th percentiles are logarithmically interpolated
    /// between the combined 5th and 95th boundaries when both are positive, otherwise
    /// linearly interpolated. Null before height 210,000, where calibration starts.
    pub prices: RarityPercentiles<Price<PerBlock<Cents, M>>>,
    /// Signed count of combined extreme boundaries crossed by spot price.
    /// Negative values mean spot is below lower bands and therefore unusually
    /// low; positive values mean it is above upper bands and unusually high;
    /// zero means neither side is crossed. It equals upper boundaries exceeded
    /// minus lower boundaries not reached and ranges from -5 through 5. Null before
    /// height 210,000.
    pub level: PerBlock<Score, M>,
    /// Agreement score across the meter's component models. More negative
    /// values mean more models identify a rare low valuation; more positive
    /// values mean more models identify a rare high valuation. It sums each
    /// component's rarity level, from -5 through 5. Null before height 210,000.
    pub score: PerBlock<Score, M>,
}

const VERSION: Version = Version::new(3);

impl RarityMeterInner {
    pub(crate) fn import(
        db: &Database,
        prefix: &str,
        version: Version,
        mappings: &IndexSources,
    ) -> Result<Self> {
        let version = version + VERSION;
        let prices = RarityPercentiles::try_from_fn(|id| {
            Price::import(db, &format!("{prefix}_{}", id.suffix()), version, mappings)
        })?;

        Ok(RarityMeterInner {
            prices,
            level: PerBlock::import(db, &format!("{prefix}_level"), version, mappings)?,
            // Rebuild scores using capped totals instead of overflowing i8 sums.
            score: PerBlock::import(
                db,
                &format!("{prefix}_score"),
                version + Version::ONE,
                mappings,
            )?,
        })
    }

    /// Local and Cycle: prices from their components' boundaries, and scores summing each
    /// component's level.
    pub(crate) fn compute(
        &mut self,
        components: &[&Component],
        spot: &impl ReadableVec<Height, Cents>,
        starting_height: Height,
        exit: &Exit,
    ) -> Result<()> {
        let version = boundary_version(components);
        let prices_end = components
            .iter()
            .map(|component| component::boundary_len(component))
            .min()
            .unwrap_or_default();
        self.compute_rows(
            spot,
            starting_height,
            (version, prices_end),
            (version + spot.version(), prices_end.min(spot.len())),
            |range| {
                components
                    .iter()
                    .map(|component| {
                        component::collect_boundary_prices(component, range.start, range.end)
                    })
                    .collect()
            },
            |range, bands, spot| {
                (0..range.len())
                    .map(|offset| {
                        Self::total_score(
                            bands
                                .iter()
                                .map(|bands| i16::from(Self::score_at(spot[offset], bands, offset)))
                                .sum(),
                        )
                    })
                    .collect()
            },
            exit,
        )
    }

    /// Full: prices from Local's and Cycle's boundaries, and scores summing theirs.
    pub(crate) fn compute_combined(
        &mut self,
        meters: &[&RarityMeterInner],
        spot: &impl ReadableVec<Height, Cents>,
        starting_height: Height,
        exit: &Exit,
    ) -> Result<()> {
        let prices_version = meters.iter().map(|meter| meter.prices_version()).sum();
        let prices_end = meters
            .iter()
            .map(|meter| meter.prices_len())
            .min()
            .unwrap_or_default();
        let score_version = meters
            .iter()
            .map(|meter| meter.score.height.version())
            .sum();
        let score_end = meters
            .iter()
            .map(|meter| meter.score.height.len())
            .min()
            .unwrap_or_default();
        self.compute_rows(
            spot,
            starting_height,
            (prices_version, prices_end),
            (score_version, score_end),
            |range| {
                meters
                    .iter()
                    .map(|meter| {
                        meter.prices.boundary_refs().map(|price| {
                            price.cents.height.collect_range_at(range.start, range.end)
                        })
                    })
                    .collect()
            },
            |range, _, _| {
                let scores: Vec<_> = meters
                    .iter()
                    .map(|meter| meter.score.height.collect_range_at(range.start, range.end))
                    .collect();
                (0..range.len())
                    .map(|offset| {
                        Self::total_score(
                            scores.iter().map(|scores| i16::from(*scores[offset])).sum(),
                        )
                    })
                    .collect()
            },
            exit,
        )
    }

    fn prices_len(&self) -> usize {
        self.prices
            .iter()
            .map(|p| p.cents.height.len())
            .min()
            .unwrap_or_default()
    }

    fn prices_version(&self) -> Version {
        self.prices.iter().map(|p| p.cents.height.version()).sum()
    }

    /// One pass pushing the prices, level and score, each from where it resumes and through its
    /// end (`prices` and `score` give their versions and ends; the level follows the prices and
    /// spot). `boundaries` gives each source's ten boundary prices over a range, `scores` the
    /// scores over a range from those and spot. Before calibration starts every value is
    /// undefined and nothing is read.
    #[allow(clippy::too_many_arguments)]
    fn compute_rows(
        &mut self,
        spot: &impl ReadableVec<Height, Cents>,
        starting_height: Height,
        (prices_version, prices_end): (Version, usize),
        (score_version, score_end): (Version, usize),
        boundaries: impl Fn(Range<usize>) -> Vec<[Vec<Cents>; 10]>,
        scores: impl Fn(Range<usize>, &[[Vec<Cents>; 10]], &[Cents]) -> Vec<Score>,
        exit: &Exit,
    ) -> Result<()> {
        debug_assert!(score_end <= prices_end, "scores past the prices");
        let max_from = usize::from(starting_height);
        let prices_start = prepare_computed(
            self.prices.iter_any_stored_mut().collect::<Vec<_>>(),
            prices_version,
            max_from.min(prices_end),
            exit,
        )?;
        let level_end = prices_end.min(spot.len());
        let level_version = self.prices_version() + spot.version();
        let level_start = prepare_computed(
            [&mut self.level.height],
            level_version,
            max_from.min(level_end),
            exit,
        )?;
        let score_start = prepare_computed(
            [&mut self.score.height],
            score_version,
            max_from.min(score_end),
            exit,
        )?;
        let mut start = prices_start;
        if level_start < level_end {
            start = start.min(level_start);
        }
        if score_start < score_end {
            start = start.min(score_start);
        }
        let levels = level_start..level_end;
        let scored = score_start..score_end;
        for chunk in write_chunks(start..prices_end) {
            let calibrated = chunk.start.max(START_HEIGHT).min(chunk.end);
            for height in chunk.start..calibrated {
                if height >= prices_start {
                    for price in self.prices.iter_mut() {
                        price.cents.height.push(Cents::NAN);
                    }
                }
                if levels.contains(&height) {
                    self.level.height.push(Score::NAN);
                }
                if scored.contains(&height) {
                    self.score.height.push(Score::NAN);
                }
            }
            if calibrated < chunk.end {
                let bands = boundaries(calibrated..chunk.end);
                let spot =
                    spot.collect_range_at(calibrated, chunk.end.min(level_end).max(calibrated));
                let scores = scores(
                    calibrated..chunk.end.min(score_end).max(calibrated),
                    &bands,
                    &spot,
                );
                for offset in 0..chunk.end - calibrated {
                    let height = calibrated + offset;
                    let values = Self::combine_percentiles(&bands, offset);
                    if height >= prices_start {
                        for (price, value) in self.prices.iter_mut().zip(values) {
                            price.cents.height.push(value);
                        }
                    }
                    if levels.contains(&height) {
                        self.level.height.push(Self::level(spot[offset], &values));
                    }
                    if scored.contains(&height) {
                        self.score.height.push(scores[offset]);
                    }
                }
            }
            write_locked(self.iter_any_stored_mut(), exit)?;
        }
        Ok(())
    }

    /// Upper boundaries spot exceeds minus lower boundaries it stays under.
    fn level(close: Cents, values: &[Cents; RARITY_PERCENTILES_LEN]) -> Score {
        let lower = RarityPercentileId::BOUNDARIES[..5]
            .iter()
            .filter(|&&id| !values[id as usize].is_nan() && close < values[id as usize])
            .count() as i8;
        let upper = RarityPercentileId::BOUNDARIES[5..]
            .iter()
            .filter(|&&id| close > values[id as usize])
            .count() as i8;
        Score::new(upper - lower)
    }

    pub fn needs_compute(
        &self,
        components: &[&Component],
        spot: &impl ReadableVec<Height, Cents>,
        starting_height: Height,
    ) -> bool {
        let prices_end = components
            .iter()
            .map(|component| component::boundary_len(component))
            .min()
            .unwrap_or_default();
        let score_end = prices_end.min(spot.len());
        let starting_height = usize::from(starting_height);

        let version = boundary_version(components);
        self.prices.iter().any(|price| {
            let target = &price.cents.height;
            target.header().computed_version() != target.header().vec_version() + version
        }) || self.level.height.header().computed_version()
            != self.level.height.header().vec_version() + self.prices_version() + spot.version()
            || self.score.height.header().computed_version()
                != self.score.height.header().vec_version() + version + spot.version()
            || self.prices_len() != prices_end
            || self.level.height.len() != score_end
            || self.score.height.len() != score_end
            || self.prices_len() > starting_height
            || self.level.height.len() > starting_height
            || self.score.height.len() > starting_height
    }

    /// 16 components score at most 80 either way, above the undefined minimum.
    fn total_score(value: i16) -> Score {
        debug_assert!(
            value.unsigned_abs() <= 80,
            "score {value} beyond 16 components"
        );
        Score::new(value as i8)
    }

    fn combine_percentiles(
        component_prices: &[[Vec<Cents>; 10]],
        offset: usize,
    ) -> [Cents; RARITY_PERCENTILES_LEN] {
        let boundary_values = RarityPercentileId::BOUNDARIES.map(|id| {
            let index = id.boundary_index().expect("boundary percentile");
            let values = component_prices
                .iter()
                .map(|component| component[index][offset]);
            if id.is_lower_boundary() {
                values
                    .filter(|value| !value.is_nan())
                    .max()
                    .unwrap_or(Cents::NAN)
            } else {
                values.min().expect("rarity meter component")
            }
        });
        let lower = boundary_values[RarityPercentileId::Pct5.boundary_index().unwrap()];
        let upper = boundary_values[RarityPercentileId::Pct95.boundary_index().unwrap()];

        RarityPercentileId::from_fn(|id| {
            id.boundary_index()
                .map(|index| boundary_values[index])
                .unwrap_or_else(|| Self::interpolate(lower, upper, id.percentile()))
        })
    }

    fn interpolate(lower: Cents, upper: Cents, percentile: f64) -> Cents {
        let position = (percentile - 0.05) / 0.90;
        let lower = f64::from(lower);
        let upper = f64::from(upper);
        let value = if lower > 0.0 && upper > 0.0 {
            (lower.ln() + position * (upper.ln() - lower.ln())).exp()
        } else {
            lower + position * (upper - lower)
        };
        Cents::from(value.round())
    }

    fn score_at(price: Cents, bands: &[Vec<Cents>; 10], index: usize) -> i8 {
        let lower = bands[..5]
            .iter()
            .filter(|band| !band[index].is_nan() && price < band[index])
            .count() as i8;
        let upper = bands[5..].iter().filter(|band| price > band[index]).count() as i8;

        upper - lower
    }
}

fn boundary_version(components: &[&Component]) -> Version {
    components
        .iter()
        .map(|c| component::boundary_version(c))
        .sum()
}
