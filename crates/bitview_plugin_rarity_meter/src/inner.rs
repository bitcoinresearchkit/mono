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
    /// crosses that boundary for any model. Where models disagree, one calling a price
    /// rarely low and another rarely high, that range reads as neither: lower boundaries
    /// never exceed the lowest 95th-percentile boundary and upper boundaries never fall
    /// below the highest 5th-percentile one, so the bands stay ordered. The 10th through
    /// 90th percentiles are logarithmically interpolated between the combined 5th and 95th
    /// boundaries when both are positive, otherwise linearly interpolated. Null before
    /// height 210,000, where calibration starts.
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

const VERSION: Version = Version::new(4);

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
            score: PerBlock::import(
                db,
                &format!("{prefix}_score"),
                version + Version::ONE,
                mappings,
            )?,
        })
    }

    /// One pass pushing the prices (from the components' boundaries), the level (spot against
    /// them) and the score (summing each component's level), each from where it resumes. Before
    /// calibration starts every value is undefined and nothing is read.
    pub(crate) fn compute(
        &mut self,
        components: &[&Component],
        spot: &impl ReadableVec<Height, Cents>,
        starting_height: Height,
        exit: &Exit,
    ) -> Result<()> {
        let version = boundary_version(components);
        let prices_end = prices_end(components);
        let scores_end = prices_end.min(spot.len());
        let max_from = usize::from(starting_height);
        let prices_start = prepare_computed(
            self.prices.iter_any_stored_mut().collect::<Vec<_>>(),
            version,
            max_from.min(prices_end),
            exit,
        )?;
        let level_version = self.prices_version() + spot.version();
        let level_start = prepare_computed(
            [&mut self.level.height],
            level_version,
            max_from.min(scores_end),
            exit,
        )?;
        let score_start = prepare_computed(
            [&mut self.score.height],
            version + spot.version(),
            max_from.min(scores_end),
            exit,
        )?;
        let mut start = prices_start;
        if level_start < scores_end {
            start = start.min(level_start);
        }
        if score_start < scores_end {
            start = start.min(score_start);
        }
        let levels = level_start..scores_end;
        let scored = score_start..scores_end;
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
                let bands: Vec<_> = components
                    .iter()
                    .map(|component| {
                        component::collect_boundary_prices(component, calibrated, chunk.end)
                    })
                    .collect();
                let spot =
                    spot.collect_range_at(calibrated, chunk.end.min(scores_end).max(calibrated));
                for (offset, height) in (calibrated..chunk.end).enumerate() {
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
                        let total = bands
                            .iter()
                            .map(|bands| i16::from(Self::score_at(spot[offset], bands, offset)))
                            .sum();
                        self.score.height.push(Self::total_score(total));
                    }
                }
            }
            write_locked(self.iter_any_stored_mut(), exit)?;
        }
        Ok(())
    }

    fn prices_len(&self) -> usize {
        self.prices
            .iter()
            .map(|p| p.cents.height.len())
            .min()
            .expect("rarity meter percentiles")
    }

    fn prices_version(&self) -> Version {
        self.prices.iter().map(|p| p.cents.height.version()).sum()
    }

    fn level(close: Cents, values: &[Cents; RARITY_PERCENTILES_LEN]) -> Score {
        Score::new(Self::boundary_score(close, |i| {
            values[RarityPercentileId::BOUNDARIES[i] as usize]
        }))
    }

    /// Upper boundaries `price` exceeds minus lower boundaries it stays under, `boundary(i)`
    /// being the `i`th of the five lower then five upper ones. An undefined lower boundary
    /// counts as neither.
    #[inline]
    fn boundary_score(price: Cents, boundary: impl Fn(usize) -> Cents) -> i8 {
        let lower = (0..5)
            .map(&boundary)
            .filter(|&value| !value.is_nan() && price < value)
            .count() as i8;
        let upper = (5..10).filter(|&i| price > boundary(i)).count() as i8;
        upper - lower
    }

    pub fn needs_compute(
        &self,
        components: &[&Component],
        spot: &impl ReadableVec<Height, Cents>,
        starting_height: Height,
    ) -> bool {
        let prices_end = prices_end(components);
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
        // A source's boundaries are all undefined or none.
        debug_assert_eq!(lower.is_nan(), upper.is_nan(), "half-defined boundaries");

        // Where models disagree (one's 5% boundary above another's 95%), the range between reads
        // as neither: lower boundaries are capped at the 95% one and upper floored at the 5% one.
        RarityPercentileId::from_fn(|id| match id.boundary_index() {
            Some(index) if id.is_lower_boundary() => boundary_values[index].min(upper),
            Some(index) => boundary_values[index].max(lower),
            None => Self::interpolate(lower.min(upper), upper.max(lower), id.percentile()),
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
        Cents::from(value)
    }

    fn score_at(price: Cents, bands: &[Vec<Cents>; 10], index: usize) -> i8 {
        Self::boundary_score(price, |i| bands[i][index])
    }
}

fn prices_end(components: &[&Component]) -> usize {
    components
        .iter()
        .map(|c| component::boundary_len(c))
        .min()
        .expect("rarity meter components")
}

fn boundary_version(components: &[&Component]) -> Version {
    components
        .iter()
        .map(|c| component::boundary_version(c))
        .sum()
}
