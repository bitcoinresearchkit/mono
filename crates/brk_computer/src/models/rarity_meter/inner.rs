use brk_error::Result;
use brk_indexer::Indexer;
use brk_traversable::Traversable;
use brk_types::{Cents, Height, RARITY_PERCENTILES_LEN, RarityPercentileId, StoredI8, Version};
use vecdb::{AnyVec, ColumnId, Database, Exit, ReadableVec, Rw, StorageMode, WritableVec};

use crate::{
    indexes,
    internal::{ColumnarPerBlock, LazyColumnPerBlock, PerBlock, Price},
};

use super::{COMPUTE_BATCH_SIZE, Component, percentiles::RarityPercentiles};

#[derive(Traversable)]
pub struct RarityMeterInner<M: StorageMode = Rw> {
    #[traversable(flatten)]
    pub prices: ColumnarPerBlock<
        Cents,
        RarityPercentileId,
        RarityPercentiles<Price<LazyColumnPerBlock<Cents, RarityPercentileId>>>,
        M,
    >,
    pub index: PerBlock<StoredI8, M>,
    pub score: PerBlock<StoredI8, M>,
}

const VERSION: Version = Version::ONE;

impl RarityMeterInner {
    pub(crate) fn forced_import(
        db: &Database,
        prefix: &str,
        version: Version,
        indexes: &indexes::Vecs,
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
                        indexes,
                    )
                })
            },
        )?;

        Ok(Self {
            prices,
            index: PerBlock::forced_import(db, &format!("{prefix}_index"), version, indexes)?,
            score: PerBlock::forced_import(db, &format!("{prefix}_score"), version, indexes)?,
        })
    }

    pub(super) fn compute(
        &mut self,
        components: &[&Component],
        spot: &impl ReadableVec<Height, Cents>,
        indexer: &Indexer,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let dependency_version = components
            .iter()
            .map(|component| component.boundary_version())
            .sum();
        let source_end = components
            .iter()
            .map(|component| component.boundary_len())
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
                    .map(|component| component.collect_boundary_prices(range.start, range.end))
                    .collect();

                for offset in 0..range.len() {
                    cents.push(Self::combine_percentiles(&component_prices, offset));
                }

                Ok(())
            },
            exit,
        )?;

        self.compute_index(spot, indexer, exit)?;
        self.compute_score(components, spot, indexer, exit)
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
        spot: &impl ReadableVec<Height, Cents>,
        indexer: &Indexer,
        exit: &Exit,
    ) -> Result<()> {
        let starting_height = indexer.safe_lengths().height;
        let dependency_version = components
            .iter()
            .map(|component| component.boundary_version())
            .sum::<Version>()
            + spot.version();
        let source_end = components
            .iter()
            .map(|component| component.boundary_len())
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
                    .map(|component| component.collect_boundary_prices(range.start, range.end))
                    .collect();
                for (offset, price) in spot.into_iter().enumerate() {
                    let value = component_prices
                        .iter()
                        .map(|bands| Self::score_at(price, bands, offset))
                        .sum();
                    score.push(StoredI8::new(value));
                }

                Ok(())
            },
            exit,
        )?;

        Ok(())
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
                values.max().expect("rarity meter component")
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
        let row = RarityMeterInner::combine_percentiles(&components, 0);

        assert_eq!(*Pct0_1.get(&row), Cents::from(15_u64));
        assert_eq!(*Pct5.get(&row), Cents::from(55_u64));
        assert_eq!(*Pct95.get(&row), Cents::from(450_u64));
        assert_eq!(*Pct99_9.get(&row), Cents::from(850_u64));
        assert_eq!(
            *Pct50.get(&row),
            RarityMeterInner::interpolate(Cents::from(55_u64), Cents::from(450_u64), 0.5)
        );
    }
}
