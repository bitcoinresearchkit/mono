use std::collections::BTreeMap;

use bitview_transforms::RatioDollars;
use bitview_traversable::{Traversable, TreeNode};
use bitview_vecs::{BasisPointsPerBlock, LazyBasisPointsPerBlock};
use brk_types::{BasisPoints32, Dollars, Height, Version};
use common::{indexes, init_cache};
use tempfile::tempdir;
use vecdb::{
    AnySerializableVec, AnyStoredVec, AnyVec, BinaryTransform, Database, ReadableVec, WritableVec,
};

mod common;

#[test]
fn stored_and_lazy_views_publish_bps_not_ppm() {
    init_cache();
    let directory = tempdir().unwrap();
    let db = Database::open(directory.path()).unwrap();
    let mut indexes = indexes(&db);
    indexes.first_height.day1 = common::first_heights("bps_days", [0usize, 2, 4].map(Height::from));
    let values = [
        BasisPoints32::ZERO,
        BasisPoints32::ONE,
        BasisPoints32::from(0.123_499),
        BasisPoints32::NAN,
        BasisPoints32::MAX,
        BasisPoints32::from(74_641.0),
    ];
    let mut stored =
        BasisPointsPerBlock::forced_import(&db, "puell_multiple", Version::ONE, &indexes).unwrap();
    for value in values {
        stored.bps.height.push(value);
    }
    stored.bps.height.write().unwrap();
    let lazy = LazyBasisPointsPerBlock::from_height_source(
        "nvt",
        Version::ONE,
        &stored.bps.height,
        &indexes,
    );
    macro_rules! check {
        ($view:ident, $name:literal) => {{
            assert_eq!($view.bps.height.name(), concat!($name, "_bps"));
            assert_eq!($view.ratio.height.name(), $name);
            assert_eq!($view.bps.height.collect_range_at(0, 6), values);
            assert_eq!(
                f32::from($view.ratio.height.collect_one_at(1).unwrap()),
                1.0
            );
            assert_eq!(
                f32::from($view.ratio.height.collect_one_at(2).unwrap()),
                0.1234f32
            );
            assert!(f32::from($view.ratio.height.collect_one_at(3).unwrap()).is_nan());
            let TreeNode::Branch(branch) = $view.to_tree_node() else {
                panic!("expected ratio branch");
            };
            assert!(branch.get("ppm").is_none());
            let TreeNode::Leaf(leaf) = branch.get("bps").unwrap() else {
                panic!("expected BPS leaf");
            };
            assert_eq!(leaf.kind(), "BasisPoints32");
            let mut descriptions = BTreeMap::new();
            $view.collect_series_descriptions(&mut Vec::new(), &mut descriptions);
            assert!(
                descriptions
                    .get(concat!($name, "_bps"))
                    .unwrap()
                    .join(" ")
                    .contains("10,000")
            );
            let mut json = Vec::new();
            $view
                .bps
                .height
                .write_json(Some(0), Some(4), &mut json)
                .unwrap();
            assert_eq!(json, b"[0,10000,1234,null]");
        }};
    }
    check!(stored, "puell_multiple");
    check!(lazy, "nvt");
    assert_eq!(stored.bps.day1.collect(), lazy.bps.day1.collect());
    drop(lazy);
    drop(stored);
    let reopened =
        BasisPointsPerBlock::forced_import(&db, "puell_multiple", Version::ONE, &indexes).unwrap();
    assert_eq!(reopened.bps.height.collect_range_at(0, 6), values);
}

#[test]
fn puell_transform_floors_and_preserves_nonfinite_zero_behavior() {
    for (num, den, raw) in [
        (123.499, 100.0, 12_349),
        (53_302.002_003, 1.0, 533_020_020),
        (0.0, 1.0, 0),
        (1.0, 0.0, 0),
        (f64::NAN, 1.0, 0),
        (1.0, f64::NAN, 0),
    ] {
        assert_eq!(
            RatioDollars::<BasisPoints32>::apply(Dollars::from(num), Dollars::from(den)).inner(),
            raw
        );
    }
}
