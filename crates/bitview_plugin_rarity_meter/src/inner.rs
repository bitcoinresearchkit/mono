use std::ops::Range;

use bitview_collections::RarityPercentiles;
use bitview_compute::prepare_computed;
use bitview_traversable::Traversable;
use bitview_vecs::{IndexSources, PerBlock, Price};
use brk_error::Result;
use brk_exit::Exit;
use brk_types::{Cents, Height, RARITY_PERCENTILES_LEN, RarityPercentileId, StoredI8, Version};
use vecdb::{AnyStoredVec, AnyVec, Database, ReadableVec, Rw, StorageMode, WritableVec};

use super::{COMPUTE_BATCH_SIZE, Component, component};

#[derive(Traversable)]
pub struct RarityMeterInner<M: StorageMode = Rw> {
    #[traversable(flatten)]
    /// Consensus historical price boundaries across the meter's reference
    /// models. Lower-percentile bands represent unusually low spot valuations;
    /// upper-percentile bands represent unusually high valuations. To require
    /// agreement between models, lower boundaries from 0.1% through 5% use the
    /// highest corresponding component boundary and upper boundaries from 95%
    /// through 99.9% use the lowest. The 10th through 90th percentiles are
    /// logarithmically interpolated between the combined 5th and 95th boundaries
    /// when both are positive, otherwise linearly interpolated.
    pub prices: RarityPercentiles<Price<PerBlock<Cents, M>>>,
    /// Signed count of combined extreme boundaries crossed by spot price.
    /// Negative values mean spot is below lower bands and therefore unusually
    /// low; positive values mean it is above upper bands and unusually high;
    /// zero means neither side is crossed. It equals upper boundaries exceeded
    /// minus lower boundaries not reached and ranges from -5 through 5.
    pub index: PerBlock<StoredI8, M>,
    /// Agreement score across the meter's component models. More negative
    /// values mean more models identify a rare low valuation; more positive
    /// values mean more models identify a rare high valuation. It sums each
    /// component's rarity index: two-tailed ratio components contribute from -5
    /// through 5 and lower-only direct components from -5 through 0. The total
    /// is capped to the storage range of -128 through 127.
    pub score: PerBlock<StoredI8, M>,
}

const VERSION: Version = Version::TWO;

impl RarityMeterInner {
    pub(crate) fn import(
        db: &Database,
        prefix: &str,
        version: Version,
        mappings: &IndexSources,
    ) -> Result<Self> {
        let version = version + VERSION;
        let prices = RarityPercentiles::try_from_fn(|id| {
            Price::import(
                db,
                &format!("{prefix}_{}", id.price_suffix()),
                version,
                mappings,
            )
        })?;

        Ok(RarityMeterInner {
            prices,
            index: PerBlock::import(db, &format!("{prefix}_index"), version, mappings)?,
            // Rebuild scores using capped totals instead of overflowing i8 sums.
            score: PerBlock::import(
                db,
                &format!("{prefix}_score"),
                version + Version::ONE,
                mappings,
            )?,
        })
    }

    pub(crate) fn compute(
        &mut self,
        components: &[&Component],
        lower_components: &[[&impl ReadableVec<Height, Cents>; 5]],
        spot: &impl ReadableVec<Height, Cents>,
        starting_height: Height,
        exit: &Exit,
    ) -> Result<()> {
        let dependency_version = boundary_version(components, lower_components);
        let source_end = components
            .iter()
            .map(|component| component::boundary_len(component))
            .chain(lower_components.iter().flatten().map(|band| band.len()))
            .min()
            .unwrap_or_default();

        let prices_start = prepare_computed(
            self.prices
                .iter_mut()
                .map(|p| &mut p.cents.height as &mut dyn AnyStoredVec)
                .collect::<Vec<_>>(),
            dependency_version,
            usize::from(starting_height).min(source_end),
            exit,
        )?;
        let score_end = source_end.min(spot.len());
        let index_version = self.prices_version() + spot.version();
        let index_start = prepare_computed(
            [&mut self.index.height],
            index_version,
            usize::from(starting_height).min(score_end),
            exit,
        )?;
        let score_start = prepare_computed(
            [&mut self.score.height],
            dependency_version + spot.version(),
            usize::from(starting_height).min(score_end),
            exit,
        )?;
        let mut start = prices_start;
        if index_start < score_end {
            start = start.min(index_start);
        }
        if score_start < score_end {
            start = start.min(score_start);
        }
        while start < source_end {
            let end = (start + COMPUTE_BATCH_SIZE).min(source_end);
            let component_prices: Vec<_> = components
                .iter()
                .map(|component| component::collect_boundary_prices(component, start, end))
                .collect();
            let lower_prices: Vec<_> = lower_components
                .iter()
                .map(|bands| {
                    bands
                        .each_ref()
                        .map(|band| band.collect_range_at(start, end))
                })
                .collect();
            let spot = spot.collect_range_at(start, end.min(score_end));
            for offset in 0..end - start {
                let height = start + offset;
                let values =
                    RarityMeterInner::combine_percentiles(&component_prices, &lower_prices, offset);
                if height >= prices_start {
                    for (price, value) in self.prices.iter_mut().zip(values) {
                        price.cents.height.push(value);
                    }
                }
                if let Some(&close) = spot.get(offset) {
                    if height >= index_start {
                        let lower = RarityPercentileId::BOUNDARIES[..5]
                            .iter()
                            .filter(|&&id| close < values[id as usize])
                            .count() as i8;
                        let upper = RarityPercentileId::BOUNDARIES[5..]
                            .iter()
                            .filter(|&&id| close > values[id as usize])
                            .count() as i8;
                        self.index.height.push(StoredI8::new(upper - lower));
                    }
                    if height >= score_start {
                        let score = component_prices
                            .iter()
                            .map(|bands| {
                                i16::from(RarityMeterInner::score_at(close, bands, offset))
                            })
                            .chain(lower_prices.iter().map(|bands| {
                                i16::from(RarityMeterInner::lower_score_at(close, bands, offset))
                            }))
                            .sum();
                        self.score
                            .height
                            .push(RarityMeterInner::capped_score(score));
                    }
                }
            }
            let _lock = exit.lock();
            for price in self.prices.iter_mut() {
                price.cents.height.write()?;
            }
            self.index.height.write()?;
            self.score.height.write()?;
            start = end;
        }
        Ok(())
    }

    pub(crate) fn compute_combined(
        &mut self,
        meters: &[&RarityMeterInner],
        spot: &impl ReadableVec<Height, Cents>,
        starting_height: Height,
        exit: &Exit,
    ) -> Result<()> {
        let dependency_version = meters.iter().map(|meter| meter.prices_version()).sum();
        let source_end = meters
            .iter()
            .map(|meter| meter.prices_len())
            .min()
            .unwrap_or_default();

        self.compute_prices(
            starting_height,
            source_end,
            dependency_version,
            |range| {
                let meter_prices: Vec<_> = meters
                    .iter()
                    .map(|meter| {
                        meter.prices.boundary_refs().map(|price| {
                            price.cents.height.collect_range_at(range.start, range.end)
                        })
                    })
                    .collect();

                (0..range.len())
                    .map(|offset| RarityMeterInner::combine_percentiles(&meter_prices, &[], offset))
                    .collect()
            },
            exit,
        )?;

        self.compute_index(spot, starting_height, exit)?;
        self.compute_combined_score(meters, starting_height, exit)
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
    fn compute_prices(
        &mut self,
        starting_height: Height,
        source_end: usize,
        version: Version,
        mut compute: impl FnMut(Range<usize>) -> Vec<[Cents; RARITY_PERCENTILES_LEN]>,
        exit: &Exit,
    ) -> Result<()> {
        let start = prepare_computed(
            self.prices
                .iter_mut()
                .map(|price| &mut price.cents.height as &mut dyn AnyStoredVec)
                .collect::<Vec<_>>(),
            version,
            usize::from(starting_height).min(source_end),
            exit,
        )?;
        let mut chunk_start = start;
        while chunk_start < source_end {
            let end = (chunk_start + COMPUTE_BATCH_SIZE).min(source_end);
            for values in compute(chunk_start..end) {
                for (price, value) in self.prices.iter_mut().zip(values) {
                    price.cents.height.push(value);
                }
            }
            let _lock = exit.lock();
            for price in self.prices.iter_mut() {
                price.cents.height.write()?;
            }
            chunk_start = end;
        }
        Ok(())
    }

    pub fn needs_compute(
        &self,
        components: &[&Component],
        lower_components: &[[&impl ReadableVec<Height, Cents>; 5]],
        spot: &impl ReadableVec<Height, Cents>,
        starting_height: Height,
    ) -> bool {
        let prices_end = components
            .iter()
            .map(|component| component::boundary_len(component))
            .chain(lower_components.iter().flatten().map(|band| band.len()))
            .min()
            .unwrap_or_default();
        let score_end = prices_end.min(spot.len());
        let starting_height = usize::from(starting_height);

        let version = boundary_version(components, lower_components);
        self.prices.iter().any(|price| {
            let target = &price.cents.height;
            target.header().computed_version() != target.header().vec_version() + version
        }) || self.index.height.header().computed_version()
            != self.index.height.header().vec_version() + self.prices_version() + spot.version()
            || self.score.height.header().computed_version()
                != self.score.height.header().vec_version() + version + spot.version()
            || self.prices_len() != prices_end
            || self.index.height.len() != score_end
            || self.score.height.len() != score_end
            || self.prices_len() > starting_height
            || self.index.height.len() > starting_height
            || self.score.height.len() > starting_height
    }

    fn compute_index(
        &mut self,
        spot: &impl ReadableVec<Height, Cents>,
        starting_height: Height,
        exit: &Exit,
    ) -> Result<()> {
        let bands = self.prices.boundary_refs().map(|price| &price.cents.height);
        let source_end = bands
            .iter()
            .map(|band| band.len())
            .min()
            .unwrap_or_default()
            .min(spot.len());

        let version = self.prices_version() + spot.version();
        self.index.height.compute_batched_to(
            starting_height,
            source_end,
            version,
            COMPUTE_BATCH_SIZE,
            |index, range| {
                let spot = spot.collect_range_at(range.start, range.end);
                let bands = bands
                    .each_ref()
                    .map(|band| band.collect_range_at(range.start, range.end));
                for (offset, price) in spot.into_iter().enumerate() {
                    index.push(StoredI8::new(Self::score_at(price, &bands, offset)));
                }

                Ok(())
            },
            exit,
        )?;

        Ok(())
    }

    fn compute_combined_score(
        &mut self,
        meters: &[&RarityMeterInner],
        starting_height: Height,
        exit: &Exit,
    ) -> Result<()> {
        let dependency_version = meters
            .iter()
            .map(|meter| meter.score.height.version())
            .sum();
        let source_end = meters
            .iter()
            .map(|meter| meter.score.height.len())
            .min()
            .unwrap_or_default();

        self.score.height.compute_batched_to(
            starting_height,
            source_end,
            dependency_version,
            COMPUTE_BATCH_SIZE,
            |score, range| {
                let meter_scores: Vec<_> = meters
                    .iter()
                    .map(|meter| meter.score.height.collect_range_at(range.start, range.end))
                    .collect();
                for offset in 0..range.len() {
                    score.push(Self::capped_score(
                        meter_scores
                            .iter()
                            .map(|scores| i16::from(*scores[offset]))
                            .sum(),
                    ));
                }

                Ok(())
            },
            exit,
        )?;

        Ok(())
    }

    fn capped_score(value: i16) -> StoredI8 {
        StoredI8::new(value.clamp(i16::from(i8::MIN), i16::from(i8::MAX)) as i8)
    }

    fn combine_percentiles(
        component_prices: &[[Vec<Cents>; 10]],
        lower_component_prices: &[[Vec<Cents>; 5]],
        offset: usize,
    ) -> [Cents; RARITY_PERCENTILES_LEN] {
        let boundary_values = RarityPercentileId::BOUNDARIES.map(|id| {
            let index = id.boundary_index().expect("boundary percentile");
            let values = component_prices
                .iter()
                .map(|component| component[index][offset]);
            if id.is_lower_boundary() {
                lower_component_prices
                    .iter()
                    .map(|component| component[index][offset])
                    .filter(|value| !value.is_nan())
                    .chain(values)
                    .max()
                    .expect("rarity meter component")
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
        let lower = bands[..5].iter().filter(|band| price < band[index]).count() as i8;
        let upper = bands[5..].iter().filter(|band| price > band[index]).count() as i8;

        upper - lower
    }

    fn lower_score_at(price: Cents, bands: &[Vec<Cents>; 5], index: usize) -> i8 {
        -(bands
            .iter()
            .map(|band| band[index])
            .filter(|band| !band.is_nan() && price < *band)
            .count() as i8)
    }
}

fn boundary_version(
    components: &[&Component],
    lower_components: &[[&impl ReadableVec<Height, Cents>; 5]],
) -> Version {
    components
        .iter()
        .map(|c| component::boundary_version(c))
        .sum::<Version>()
        + lower_components
            .iter()
            .flatten()
            .map(|band| band.version())
            .sum::<Version>()
}
