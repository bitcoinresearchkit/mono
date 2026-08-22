use brk_error::Result;

use bitview_plugin_indexer::Indexer;
use bitview_traversable::Traversable;
use brk_types::{Cents, Height, RARITY_PERCENTILES_LEN, RarityPercentileId, StoredI8, Version};
use vecdb::{AnyVec, ColumnId, Database, Exit, ReadableVec, Rw, StorageMode, WritableVec};

use bitview_compute::{ColumnarPerBlock, LazyColumnPerBlock, PerBlock, Price};

use super::{COMPUTE_BATCH_SIZE, Component, component, percentiles::RarityPercentiles};

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
    pub prices: ColumnarPerBlock<
        Cents,
        RarityPercentileId,
        RarityPercentiles<Price<LazyColumnPerBlock<Cents, RarityPercentileId>>>,
        M,
    >,
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
    /// through 5 and lower-only direct components from -5 through 0.
    pub score: PerBlock<StoredI8, M>,
}

const VERSION: Version = Version::ONE;

pub fn forced_import(
    db: &Database,
    prefix: &str,
    version: Version,
    mappings: &bitview_plugin_mappings::Vecs,
) -> Result<RarityMeterInner> {
    RarityMeterInner::forced_import(db, prefix, version, mappings)
}

pub fn compute(
    inner: &mut RarityMeterInner,
    components: &[&Component],
    lower_components: &[[&dyn ReadableVec<Height, Option<Cents>>; 5]],
    spot: &impl ReadableVec<Height, Cents>,
    indexer: &Indexer,
    exit: &Exit,
) -> Result<()> {
    inner.compute(components, lower_components, spot, indexer, exit)
}

pub fn compute_combined(
    inner: &mut RarityMeterInner,
    meters: &[&RarityMeterInner],
    spot: &impl ReadableVec<Height, Cents>,
    indexer: &Indexer,
    exit: &Exit,
) -> Result<()> {
    inner.compute_combined(meters, spot, indexer, exit)
}

impl RarityMeterInner {
    fn forced_import(
        db: &Database,
        prefix: &str,
        version: Version,
        mappings: &bitview_plugin_mappings::Vecs,
    ) -> Result<Self> {
        let version = version + VERSION;
        let prices = ColumnarPerBlock::<Cents, RarityPercentileId, _>::forced_import(
            db,
            &format!("{prefix}_percentiles_cents"),
            version,
            |source| {
                RarityPercentiles::from_fn(|id| {
                    Price::from_columnar_source(
                        &format!("{prefix}_{}", id.price_suffix()),
                        version,
                        source,
                        id,
                        mappings,
                    )
                })
            },
        )?;

        Ok(Self {
            prices,
            index: PerBlock::forced_import(db, &format!("{prefix}_index"), version, mappings)?,
            score: PerBlock::forced_import(db, &format!("{prefix}_score"), version, mappings)?,
        })
    }

    fn compute(
        &mut self,
        components: &[&Component],
        lower_components: &[[&dyn ReadableVec<Height, Option<Cents>>; 5]],
        spot: &impl ReadableVec<Height, Cents>,
        indexer: &Indexer,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let dependency_version = components
            .iter()
            .map(|component| component::boundary_version(component))
            .sum::<Version>()
            + lower_components
                .iter()
                .flatten()
                .map(|band| band.version())
                .sum::<Version>();
        let source_end = components
            .iter()
            .map(|component| component::boundary_len(component))
            .chain(lower_components.iter().flatten().map(|band| band.len()))
            .min()
            .unwrap_or_default();

        self.prices.height.compute_batched_to(
            starting_height,
            source_end,
            dependency_version,
            COMPUTE_BATCH_SIZE,
            |cents, range| {
                let component_prices: Vec<_> = components
                    .iter()
                    .map(|component| {
                        component::collect_boundary_prices(component, range.start, range.end)
                    })
                    .collect();
                let lower_component_prices: Vec<_> = lower_components
                    .iter()
                    .map(|bands| {
                        bands
                            .each_ref()
                            .map(|band| band.collect_range_dyn(range.start, range.end))
                    })
                    .collect();

                for offset in 0..range.len() {
                    cents.push(Self::combine_percentiles(
                        &component_prices,
                        &lower_component_prices,
                        offset,
                    ));
                }

                Ok(())
            },
            exit,
        )?;

        self.compute_index(spot, indexer, exit)?;
        self.compute_score(components, lower_components, spot, indexer, exit)
    }

    fn compute_combined(
        &mut self,
        meters: &[&RarityMeterInner],
        spot: &impl ReadableVec<Height, Cents>,
        indexer: &Indexer,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let dependency_version = meters
            .iter()
            .map(|meter| meter.prices.height.version())
            .sum();
        let source_end = meters
            .iter()
            .map(|meter| meter.prices.height.len())
            .min()
            .unwrap_or_default();

        self.prices.height.compute_batched_to(
            starting_height,
            source_end,
            dependency_version,
            COMPUTE_BATCH_SIZE,
            |cents, range| {
                let meter_prices: Vec<_> = meters
                    .iter()
                    .map(|meter| {
                        meter.prices.boundary_refs().map(|price| {
                            price.cents.height.collect_range_at(range.start, range.end)
                        })
                    })
                    .collect();

                for offset in 0..range.len() {
                    cents.push(Self::combine_percentiles(&meter_prices, &[], offset));
                }

                Ok(())
            },
            exit,
        )?;

        self.compute_index(spot, indexer, exit)?;
        self.compute_combined_score(meters, indexer, exit)
    }

    fn compute_index(
        &mut self,
        spot: &impl ReadableVec<Height, Cents>,
        indexer: &Indexer,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let bands = self.prices.boundary_refs().map(|price| &price.cents.height);
        let source_end = bands
            .iter()
            .map(|band| band.len())
            .min()
            .unwrap_or_default()
            .min(spot.len());

        self.index.height.compute_batched_to(
            starting_height,
            source_end,
            self.prices.height.version() + spot.version(),
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

    fn compute_score(
        &mut self,
        components: &[&Component],
        lower_components: &[[&dyn ReadableVec<Height, Option<Cents>>; 5]],
        spot: &impl ReadableVec<Height, Cents>,
        indexer: &Indexer,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let dependency_version = components
            .iter()
            .map(|component| component::boundary_version(component))
            .sum::<Version>()
            + lower_components
                .iter()
                .flatten()
                .map(|band| band.version())
                .sum::<Version>()
            + spot.version();
        let source_end = components
            .iter()
            .map(|component| component::boundary_len(component))
            .chain(lower_components.iter().flatten().map(|band| band.len()))
            .min()
            .unwrap_or_default()
            .min(spot.len());

        self.score.height.compute_batched_to(
            starting_height,
            source_end,
            dependency_version,
            COMPUTE_BATCH_SIZE,
            |score, range| {
                let spot = spot.collect_range_at(range.start, range.end);
                let component_prices: Vec<_> = components
                    .iter()
                    .map(|component| {
                        component::collect_boundary_prices(component, range.start, range.end)
                    })
                    .collect();
                let lower_component_prices: Vec<_> = lower_components
                    .iter()
                    .map(|bands| {
                        bands
                            .each_ref()
                            .map(|band| band.collect_range_dyn(range.start, range.end))
                    })
                    .collect();
                for (offset, price) in spot.into_iter().enumerate() {
                    let value = component_prices
                        .iter()
                        .map(|bands| Self::score_at(price, bands, offset))
                        .sum::<i8>()
                        + lower_component_prices
                            .iter()
                            .map(|bands| Self::lower_score_at(price, bands, offset))
                            .sum::<i8>();
                    score.push(StoredI8::new(value));
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
        indexer: &Indexer,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
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
                    score.push(StoredI8::new(
                        meter_scores.iter().map(|scores| *scores[offset]).sum(),
                    ));
                }

                Ok(())
            },
            exit,
        )?;

        Ok(())
    }

    fn combine_percentiles(
        component_prices: &[[Vec<Cents>; 10]],
        lower_component_prices: &[[Vec<Option<Cents>>; 5]],
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
                    .filter_map(|component| component[index][offset])
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

    fn lower_score_at(price: Cents, bands: &[Vec<Option<Cents>>; 5], index: usize) -> i8 {
        -(bands
            .iter()
            .filter_map(|band| band[index])
            .filter(|band| !band.is_nan() && price < *band)
            .count() as i8)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bands(values: [u64; 10]) -> [Vec<Cents>; 10] {
        values.map(|value| vec![Cents::from(value)])
    }

    #[test]
    fn combines_tightest_boundaries_and_interpolates_inner_percentiles() {
        use RarityPercentileId::*;

        let components = [
            bands([10, 20, 30, 40, 50, 500, 600, 700, 800, 900]),
            bands([15, 25, 35, 45, 55, 450, 550, 650, 750, 850]),
        ];
        let row = RarityMeterInner::combine_percentiles(&components, &[], 0);

        assert_eq!(*Pct0_1.get(&row), Cents::from(15_u64));
        assert_eq!(*Pct5.get(&row), Cents::from(55_u64));
        assert_eq!(*Pct95.get(&row), Cents::from(450_u64));
        assert_eq!(*Pct99_9.get(&row), Cents::from(850_u64));
        assert_eq!(
            *Pct50.get(&row),
            RarityMeterInner::interpolate(Cents::from(55_u64), Cents::from(450_u64), 0.5)
        );
    }

    #[test]
    fn includes_finite_direct_lower_boundaries_only() {
        use RarityPercentileId::*;

        let components = [bands([10, 20, 30, 40, 50, 500, 600, 700, 800, 900])];
        let lower_components = [[
            vec![Some(Cents::from(15_u64))],
            vec![Some(Cents::NAN)],
            vec![None],
            vec![Some(Cents::from(45_u64))],
            vec![Some(Cents::from(55_u64))],
        ]];
        let row = RarityMeterInner::combine_percentiles(&components, &lower_components, 0);

        assert_eq!(*Pct0_1.get(&row), Cents::from(15_u64));
        assert_eq!(*Pct0_5.get(&row), Cents::from(20_u64));
        assert_eq!(*Pct1.get(&row), Cents::from(30_u64));
        assert_eq!(*Pct2.get(&row), Cents::from(45_u64));
        assert_eq!(*Pct5.get(&row), Cents::from(55_u64));
        assert_eq!(*Pct95.get(&row), Cents::from(500_u64));
        assert_eq!(
            RarityMeterInner::lower_score_at(Cents::from(35_u64), &lower_components[0], 0),
            -2
        );
    }
}
