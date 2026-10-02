use bitview_traversable::{Traversable, TreeNode};
use bitview_vecs::PercentPerBlock;
use brk_types::{PartsPerMillionSigned32, PartsPerMillionSigned64, Version};
use common::{indexes, init_cache};
use tempfile::tempdir;
use vecdb::{AnySerializableVec, AnyStoredVec, AnyVec, Database, ReadableVec, WritableVec};

mod common;

#[test]
fn signed_ppm_width_migration_rebuilds_storage_and_preserves_public_units() {
    init_cache();
    let directory = tempdir().unwrap();
    let name = "hash_price_rebound";
    let db = Database::open(directory.path()).unwrap();
    let sources = indexes(&db);
    let mut old = PercentPerBlock::<PartsPerMillionSigned64>::forced_import(
        &db,
        name,
        Version::ONE,
        &sources,
    )
    .unwrap();
    old.ppm.height.push(PartsPerMillionSigned64::ONE);
    old.ppm.height.write().unwrap();
    drop(old);
    drop(sources);
    drop(db);

    // A schema upgrade happens on restart, after old lazy views are gone.
    let db = Database::open(directory.path()).unwrap();
    let indexes = indexes(&db);

    let mut view = PercentPerBlock::<PartsPerMillionSigned32>::forced_import(
        &db,
        name,
        Version::TWO,
        &indexes,
    )
    .unwrap();
    assert_eq!(view.ppm.height.len(), 0);
    let values = [
        PartsPerMillionSigned32::from(-0.125),
        PartsPerMillionSigned32::ONE,
        PartsPerMillionSigned32::NAN,
    ];
    for value in values {
        view.ppm.height.push(value);
    }
    view.ppm.height.write().unwrap();
    assert_eq!(view.ppm.height.name(), format!("{name}_ppm"));
    assert_eq!(view.ratio.height.name(), format!("{name}_ratio"));
    assert_eq!(view.percent.height.name(), name);
    assert_eq!(
        f32::from(view.ratio.height.collect_one_at(0).unwrap()),
        -0.125
    );
    assert_eq!(
        f32::from(view.percent.height.collect_one_at(0).unwrap()),
        -12.5
    );
    let TreeNode::Branch(branch) = view.to_tree_node() else {
        panic!("expected branch")
    };
    let TreeNode::Leaf(leaf) = branch.get("ppm").unwrap() else {
        panic!("expected ppm leaf")
    };
    assert_eq!(leaf.kind(), "PartsPerMillionSigned32");
    let mut json = Vec::new();
    view.ppm
        .height
        .write_json(Some(0), Some(3), &mut json)
        .unwrap();
    assert_eq!(json, b"[-125000,1000000,null]");
    drop(view);
    let reopened = PercentPerBlock::<PartsPerMillionSigned32>::forced_import(
        &db,
        name,
        Version::TWO,
        &indexes,
    )
    .unwrap();
    assert_eq!(reopened.ppm.height.collect_range_at(0, 3), values);
}
