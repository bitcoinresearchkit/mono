#[cfg(test)]
use crate::test_common::init_cache;
use std::{array, ops::Range};

use bitview_plugin_indexer::Lengths;
use bitview_vecs::{DailyView, RepeatDay};
use brk_exit::Exit;
use brk_types::{
    CentsSats, CentsSquaredSats, PartsPerMillion32, RARITY_PERCENTILES, RARITY_PERCENTILES_LEN,
    Sats,
};
use tempfile::tempdir;
use vecdb::{
    AnyStoredVec, AnyVec, Budgeted, BytesVec, EagerVec, ImportableVec, PcoVec, ReadableVec,
    WritableVec,
};

use super::*;
use crate::test_common as common;

struct Pipeline {
    caps: [BytesVec<Height, CentsSats>; 4],
    capitalized_caps: [BytesVec<Height, CentsSquaredSats>; 3],
    supplies: [EagerVec<PcoVec<Height, Sats, Budgeted>>; 4],
    spot: EagerVec<PcoVec<Height, Cents, Budgeted>>,
    references: ReferencePrices,
    components: [Component; 6],
    local: RarityMeterInner,
    cycle: RarityMeterInner,
    full: RarityMeterInner,
}

impl Pipeline {
    fn import(db: &Database) -> Self {
        init_cache();
        let indexes = common::indexes(db);

        let references = ReferencePrices::forced_import(db, Version::ONE, &indexes).unwrap();
        let prices = [
            &references.under_4m,
            &references.under_6m,
            &references.over_4m,
            &references.over_6m,
            &references.under_4m_capitalized_price,
            &references.under_6m_capitalized_price,
        ];
        let components = array::from_fn(|index| {
            component::forced_import(
                db,
                &format!("component_{index}"),
                Version::ONE,
                &indexes,
                &prices[index].cents.height,
            )
            .unwrap()
        });
        let meter = |name| inner::forced_import(db, name, Version::ONE, &indexes).unwrap();
        Self {
            caps: array::from_fn(|index| {
                BytesVec::forced_import(db, &format!("cap_{index}"), Version::ONE).unwrap()
            }),
            capitalized_caps: array::from_fn(|index| {
                BytesVec::forced_import(db, &format!("capitalized_cap_{index}"), Version::ONE)
                    .unwrap()
            }),
            supplies: array::from_fn(|index| common::stored(db, &format!("supply_{index}"), [])),
            spot: common::stored(db, "spot", []),
            references,
            components,
            local: meter("local"),
            cycle: meter("cycle"),
            full: meter("full"),
        }
    }

    fn append(&mut self, rows: Range<usize>) {
        for row in rows {
            for ((cap, supply), sats) in self
                .caps
                .iter_mut()
                .zip(&mut self.supplies)
                .zip([10, 20, 1, 2])
            {
                cap.push(CentsSats::new((100 + row as u128 * 10) * sats as u128));
                supply.push(Sats::new(sats));
            }
            for (cap, sats) in self.capitalized_caps.iter_mut().zip([10, 1, 2]) {
                cap.push(CentsSquaredSats::new(
                    (100 + row as u128 * 10).pow(2) * sats,
                ));
            }
            self.spot.push(Cents::new(150 + row as u64 * 100));
        }
        for cap in &mut self.caps {
            cap.write().unwrap();
        }
        for cap in &mut self.capitalized_caps {
            cap.write().unwrap();
        }
        for supply in &mut self.supplies {
            supply.write().unwrap();
        }
        self.spot.write().unwrap();
    }

    fn compute(&mut self, starting_height: Height) {
        let exit = Exit::new();
        self.references
            .compute(
                starting_height,
                self.caps.each_ref(),
                self.supplies.each_ref(),
                self.capitalized_caps.each_ref(),
                &self.spot,
                &exit,
            )
            .unwrap();
        let prices = [
            &self.references.under_4m,
            &self.references.under_6m,
            &self.references.over_4m,
            &self.references.over_6m,
            &self.references.under_4m_capitalized_price,
            &self.references.under_6m_capitalized_price,
        ];
        for (component, price) in self.components.iter_mut().zip(prices) {
            component::compute(
                component,
                &Lengths {
                    height: starting_height,
                    ..Default::default()
                },
                &price.relative.ratio.height,
                &exit,
            )
            .unwrap();
        }
        let lower: [[&DailyView<Height, Cents, RepeatDay>; 5]; 0] = [];
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
            pipeline.caps[0].truncate_if_needed_at(3).unwrap();
            pipeline.caps[0].write().unwrap();
            pipeline.compute(Height::from(starting_height));
            pipeline.check(3);
        }
        {
            let db = Database::open(directory.path()).unwrap();
            let mut pipeline = Pipeline::import(&db);
            pipeline.check(3); // Inspect before recomputing or flushing.
            for cap in &mut pipeline.caps {
                cap.truncate_if_needed_at(3).unwrap();
            }
            for cap in &mut pipeline.capitalized_caps {
                cap.truncate_if_needed_at(3).unwrap();
            }
            for supply in &mut pipeline.supplies {
                supply.truncate_if_needed_at(3).unwrap();
            }
            pipeline.spot.truncate_if_needed_at(3).unwrap();
            pipeline.append(10..12);
            pipeline.compute(Height::from(99usize));
            pipeline.check(5);
            assert_eq!(
                pipeline.references.under_4m.cents.height.collect_one_at(4),
                Some(Cents::new(210))
            );
            assert_eq!(
                pipeline
                    .references
                    .under_4m_capitalized_price
                    .cents
                    .height
                    .collect_one_at(4),
                Some(Cents::new(210))
            );
            pipeline.caps[0].truncate_if_needed_at(0).unwrap();
            pipeline.caps[0].write().unwrap();
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
    let mut reference =
        reference_price::ReferencePrice::forced_import(&db, "reference", Version::ONE, &indexes)
            .unwrap();
    let mut spot = common::stored(
        &db,
        "spot",
        (0..START_HEIGHT)
            .map(|_| Cents::new(100))
            .chain([100, 200, 300, 900, 1000].map(Cents::new)),
    );
    for _ in 0..START_HEIGHT + 5 {
        reference.cents.height.push(Cents::new(100));
    }
    reference.cents.height.write().unwrap();
    let mut component = component::forced_import(
        &db,
        "component",
        Version::ONE,
        &indexes,
        &reference.cents.height,
    )
    .unwrap();
    let compute = |reference: &mut reference_price::ReferencePrice,
                   component: &mut Component,
                   spot: &EagerVec<PcoVec<Height, Cents, Budgeted>>,
                   height| {
        reference.compute_ratio(height, spot, &exit).unwrap();
        component::compute(
            component,
            &Lengths {
                height,
                ..Default::default()
            },
            &reference.relative.ratio.height,
            &exit,
        )
        .unwrap();
    };
    compute(&mut reference, &mut component, &spot, Height::ZERO);
    for ratio in component.ratios.iter() {
        ratio.collect();
    }
    spot.truncate_if_needed_at(START_HEIGHT + 3).unwrap();
    spot.write().unwrap();
    let resume = Height::from(START_HEIGHT + 99);
    compute(&mut reference, &mut component, &spot, resume);
    for ratio in component.ratios.iter() {
        assert_eq!(ratio.len(), START_HEIGHT + 3);
    }
    for price in [400, 500] {
        spot.push(Cents::new(price));
    }
    spot.write().unwrap();
    compute(&mut reference, &mut component, &spot, resume);
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
