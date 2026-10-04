use std::collections::{BTreeSet, HashSet};

use bitview_catalog::{SeriesLeaf, SeriesLeafWithSchema, TreeNode};
use bitview_primitives::Index;
use serde_json::json;

use crate::model::Model;

fn leaf(name: &str) -> TreeNode {
    let leaf = SeriesLeaf::new(name.into(), "Sats".into(), BTreeSet::from([Index::Height]));
    TreeNode::Leaf(SeriesLeafWithSchema::new(
        leaf,
        json!({ "type": "integer" }),
    ))
}

fn branch(children: Vec<(&str, TreeNode)>) -> TreeNode {
    TreeNode::Branch(
        children
            .into_iter()
            .map(|(key, node)| (key.to_owned(), node))
            .collect(),
    )
}

/// A wrapper nested in an identical wrapper must not make a shape contain itself.
#[test]
fn nested_identical_wrappers() {
    let tree = branch(vec![(
        "a",
        branch(vec![(
            "x",
            branch(vec![("x", branch(vec![("v", leaf("metric"))]))]),
        )]),
    )]);
    let model = Model::build(&tree);
    assert_eq!(model.shape_order().len(), model.shapes.len());
}

/// Shape names are unique, usable identifiers even when paths normalize alike, collide with
/// numeric suffixes, start with a digit or spell a reserved name.
#[test]
fn shape_names_are_unique_and_usable() {
    // Each innermost branch is its own shape (a distinct leaf key); its canonical path is its path.
    let shape = |i: usize| branch(vec![(&*format!("k{i}"), leaf(&format!("s{i}")))]);
    let tree = branch(vec![
        ("alpha_beta", branch(vec![("gamma", shape(0))])),
        ("alpha", branch(vec![("beta_gamma", shape(1))])),
        (
            "other",
            branch(vec![("gamma", shape(2)), ("beta_gamma", shape(3))]),
        ),
        ("alpha_beta_gamma2", shape(4)),
        ("0", shape(5)),
        ("true", shape(6)),
    ]);

    let reserved = BTreeSet::from(["True".to_owned()]);
    let names = Model::build(&tree).shape_names(&reserved);
    let unique: HashSet<&String> = names.iter().collect();
    assert_eq!(unique.len(), names.len(), "{names:?}");
    for name in &names {
        assert!(!reserved.contains(name), "{names:?}");
        assert!(
            name.starts_with(|c: char| c.is_ascii_alphabetic()),
            "{names:?}"
        );
        assert!(name.chars().all(|c| c.is_ascii_alphanumeric()), "{names:?}");
    }
}
