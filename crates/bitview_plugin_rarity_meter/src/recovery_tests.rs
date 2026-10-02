use bitview_plugin_indexer::Lengths;
use bitview_vecs::LazyRatioPerBlock;
use brk_exit::Exit;
use brk_types::{PartsPerMillion32, RARITY_PERCENTILES, RARITY_PERCENTILES_LEN};
use tempfile::tempdir;
use vecdb::{AnyStoredVec, AnyVec, ReadableVec, WritableVec};

use super::*;
use crate::test_common::{self as common, init_cache};

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
