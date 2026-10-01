#[cfg(test)]
use crate::test_common::init_cache;
use std::{array, ops::Range};

use bitview_plugin_indexer::Lengths;
use bitview_vecs::LazyRatioPerBlock;
use brk_exit::Exit;
use brk_types::{PartsPerMillion32, PriceRatio, RARITY_PERCENTILES, RARITY_PERCENTILES_LEN};
use tempfile::tempdir;
use vecdb::{AnyStoredVec, AnyVec, Budgeted, EagerVec, PcoVec, ReadableVec, WritableVec};

use super::*;
use crate::test_common as common;

type StoredPrice = EagerVec<PcoVec<Height, Cents, Budgeted>>;

struct Pipeline {
    references: [StoredPrice; 6],
    ratios: [LazyRatioPerBlock<PriceRatio>; 6],
    spot: StoredPrice,
    components: [Component; 6],
    local: RarityMeterInner,
    cycle: RarityMeterInner,
    full: RarityMeterInner,
}

impl Pipeline {
    fn import(db: &Database) -> Self {
        init_cache();
        let indexes = common::indexes(db);

        let references =
            array::from_fn(|index| common::stored(db, &format!("reference_{index}"), []));
        let spot = common::stored(db, "spot", []);
        let ratios = array::from_fn(|index| {
            LazyRatioPerBlock::from_price_source(
                &format!("reference_{index}_ratio"),
                Version::ONE,
                &references[index],
                &spot,
                &indexes,
            )
        });
        let components = array::from_fn(|index| {
            component::forced_import(
                db,
                &format!("component_{index}"),
                Version::ONE,
                &indexes,
                &references[index],
            )
            .unwrap()
        });
        let meter = |name| inner::forced_import(db, name, Version::ONE, &indexes).unwrap();
        Self {
            spot,
            references,
            ratios,
            components,
            local: meter("local"),
            cycle: meter("cycle"),
            full: meter("full"),
        }
    }

    fn append(&mut self, rows: Range<usize>) {
        for row in rows {
            for reference in &mut self.references {
                reference.push(Cents::new(100 + row as u64 * 10));
            }
            self.spot.push(Cents::new(150 + row as u64 * 100));
        }
        for reference in &mut self.references {
            reference.write().unwrap();
        }
        self.spot.write().unwrap();
    }

    fn compute(&mut self, starting_height: Height) {
        let exit = Exit::new();
        for (component, ratio) in self.components.iter_mut().zip(&self.ratios) {
            component::compute(
                component,
                &Lengths {
                    height: starting_height,
                    ..Default::default()
                },
                &ratio.ratio.height,
                &exit,
            )
            .unwrap();
        }
        let lower: [[&StoredPrice; 5]; 0] = [];
        inner::compute(
            &mut self.local,
            &[&self.components[0], &self.components[1]],
            &lower,
            &self.spot,
            starting_height,
            &exit,
        )
        .unwrap();
        inner::compute(
            &mut self.cycle,
            &[&self.components[2], &self.components[3]],
            &lower,
            &self.spot,
            starting_height,
            &exit,
        )
        .unwrap();
        inner::compute_combined(
            &mut self.full,
            &[&self.local, &self.cycle],
            &self.spot,
            starting_height,
            &exit,
        )
        .unwrap();
    }

    fn check(&self, len: usize) {
        for component in &self.components {
            for ratio in component.ratios.iter() {
                assert_eq!(ratio.len(), len, "component ratios");
                assert_eq!(ratio.collect().len(), len);
            }
            for band in component.bands.iter() {
                assert_eq!(band.price.cents.height.len(), len, "component bands");
                assert_eq!(
                    band.price.cents.height.collect_range_at(0, len),
                    vec![Cents::ZERO; len]
                );
            }
        }
        for (meter, score) in [(&self.local, 10), (&self.cycle, 10), (&self.full, 20)] {
            for price in meter.prices.iter() {
                assert_eq!(price.cents.height.len(), len, "meter prices");
                assert_eq!(
                    price.cents.height.collect().as_slice(),
                    vec![Cents::ZERO; len]
                );
            }
            assert_eq!(meter.index.height.len(), len, "meter index");
            assert_eq!(meter.score.height.len(), len, "meter score");
            assert!(
                meter
                    .index
                    .height
                    .collect()
                    .iter()
                    .all(|value| **value == 5)
            );
            assert!(
                meter
                    .score
                    .height
                    .collect()
                    .iter()
                    .all(|value| **value == score)
            );
        }
    }
}

#[test]
fn shortened_sources_recover_through_components_and_all_meters() {
    for starting_height in [3usize, 99] {
        let directory = tempdir().unwrap();
        {
            let db = Database::open(directory.path()).unwrap();
            let mut pipeline = Pipeline::import(&db);
            pipeline.append(0..5);
            pipeline.compute(Height::ZERO);
            pipeline.check(5); // Warm reader snapshots before truncation.
            for reference in &mut pipeline.references {
                reference.truncate_if_needed_at(3).unwrap();
                reference.write().unwrap();
            }
            pipeline.compute(Height::from(starting_height));
            pipeline.check(3);
        }
        {
            let db = Database::open(directory.path()).unwrap();
            let mut pipeline = Pipeline::import(&db);
            pipeline.check(3); // Inspect before recomputing or flushing.
            pipeline.spot.truncate_if_needed_at(3).unwrap();
            pipeline.append(10..12);
            pipeline.compute(Height::from(99usize));
            pipeline.check(5);
            for reference in &mut pipeline.references {
                reference.truncate_if_needed_at(0).unwrap();
                reference.write().unwrap();
            }
            pipeline.compute(Height::from(99usize));
            pipeline.check(0);
        }
        let db = Database::open(directory.path()).unwrap();
        Pipeline::import(&db).check(0);
    }
}

#[test]
fn shortened_component_rebuilds_percentile_state_before_appending() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let indexes = common::indexes(&db);

    let exit = Exit::new();
    let reference = common::stored(
        &db,
        "reference",
        (0..START_HEIGHT + 5).map(|_| Cents::new(100)),
    );
    let mut spot = common::stored(
        &db,
        "spot",
        (0..START_HEIGHT)
            .map(|_| Cents::new(100))
            .chain([100, 200, 300, 900, 1000].map(Cents::new)),
    );
    let ratio = LazyRatioPerBlock::from_price_source(
        "reference_ratio",
        Version::ONE,
        &reference,
        &spot,
        &indexes,
    );
    let mut component =
        component::forced_import(&db, "component", Version::ONE, &indexes, &reference).unwrap();
    let compute = |component: &mut Component, height| {
        component::compute(
            component,
            &Lengths {
                height,
                ..Default::default()
            },
            &ratio.ratio.height,
            &exit,
        )
        .unwrap();
    };
    compute(&mut component, Height::ZERO);
    for ratio in component.ratios.iter() {
        ratio.collect();
    }
    spot.truncate_if_needed_at(START_HEIGHT + 3).unwrap();
    spot.write().unwrap();
    let resume = Height::from(START_HEIGHT + 99);
    compute(&mut component, resume);
    for ratio in component.ratios.iter() {
        assert_eq!(ratio.len(), START_HEIGHT + 3);
    }
    for price in [400, 500] {
        spot.push(Cents::new(price));
    }
    spot.write().unwrap();
    compute(&mut component, resume);
    let mut expected = BlockDecayPercentiles::default();
    for offset in 0..5 {
        expected.add(START_HEIGHT + offset, (offset + 1) as f32);
        let mut quantiles = [0.0; RARITY_PERCENTILES_LEN];
        expected.quantiles(&RARITY_PERCENTILES, &mut quantiles);
        for (ratio, value) in component.ratios.iter().zip(quantiles) {
            assert_eq!(ratio.len(), START_HEIGHT + 5);
            assert_eq!(
                ratio.collect_one_at(START_HEIGHT + offset),
                Some(PartsPerMillion32::from(value))
            );
        }
    }
}
