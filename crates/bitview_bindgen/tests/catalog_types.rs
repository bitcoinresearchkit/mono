use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
};

use bitview_bindgen::{
    CatalogTree, CatalogType, CatalogValue, ClientOutputPaths, detect_index_patterns,
    generate_clients,
};
use bitview_catalog::{SeriesLeaf, SeriesLeafWithSchema, TreeBranch, TreeNode};
use brk_types::Index;
use indexmap::IndexMap;
use serde_json::json;
use tempfile::tempdir;

fn leaf(name: &str, kind: &str, index: Index) -> TreeNode {
    TreeNode::Leaf(SeriesLeafWithSchema::new(
        SeriesLeaf::new(name.into(), kind.into(), BTreeSet::from([index])),
        json!({"type": "integer"}),
    ))
}

fn family(names: [&str; 2], kind: &str, index: Index) -> TreeNode {
    let mut branch = TreeBranch {
        source: Some("fixture::Stats"),
        ..Default::default()
    };
    for (field, name) in ["min", "max"].into_iter().zip(names) {
        branch
            .merge_field(
                field.into(),
                leaf(name, kind, index),
                Some("fixture::Stats::A"),
            )
            .unwrap();
    }
    TreeNode::Branch(branch)
}

#[test]
fn declared_types_share_a_family_without_any_naming_convention() {
    let catalog = TreeNode::branch(IndexMap::from([
        (
            "first".into(),
            family(["unrelated", "max_first"], "Sats", Index::Height),
        ),
        (
            "second".into(),
            family(["é_min_usd", "another_name"], "Cents", Index::Day1),
        ),
    ]));
    let indexes = detect_index_patterns(&catalog);
    let tree = CatalogTree::from_catalog(&catalog, &indexes);
    let families: Vec<_> = tree
        .families
        .iter()
        .filter(|f| f.source == "fixture::Stats")
        .collect();
    assert_eq!(families.len(), 1);
    assert_eq!(families[0].name, "CatalogStats");
    assert_eq!(families[0].parameters, 1);
    assert_eq!(families[0].fields, [("min".into(), 0), ("max".into(), 0)]);

    let mut actual = BTreeMap::new();
    fn walk(tree: &CatalogTree, node: usize, path: String, actual: &mut BTreeMap<String, String>) {
        let binding = &tree.nodes[node];
        match (&tree.types[binding.type_id], &binding.value) {
            (CatalogType::Leaf { .. }, CatalogValue::Leaf(name)) => {
                actual.insert(path, name.clone());
            }
            (CatalogType::Branch { family, .. }, CatalogValue::Branch(children)) => {
                for ((key, _), &child) in tree.families[*family].fields.iter().zip(children) {
                    walk(tree, child, format!("{path}/{key}"), actual);
                }
            }
            _ => panic!("binding/type mismatch"),
        }
    }
    walk(&tree, tree.root, String::new(), &mut actual);
    assert_eq!(
        actual,
        BTreeMap::from([
            ("/first/min".into(), "unrelated".into()),
            ("/first/max".into(), "max_first".into()),
            ("/second/min".into(), "é_min_usd".into()),
            ("/second/max".into(), "another_name".into()),
        ])
    );
}

#[test]
fn declared_slots_and_public_projections_keep_independent_parameters() {
    for (declaration, kind, index) in [
        ("fixture::Stats::B", "Sats", Index::Height),
        ("fixture::Stats::A", "Cents", Index::Day1),
    ] {
        let mut catalog = family(["a", "b"], "Sats", Index::Height);
        let TreeNode::Branch(branch) = &mut catalog else {
            unreachable!()
        };
        branch.field_types.insert("max".into(), declaration);
        branch.children.insert("max".into(), leaf("b", kind, index));
        let tree = CatalogTree::from_catalog(&catalog, &detect_index_patterns(&catalog));
        assert_eq!(tree.families[0].parameters, 2);
    }
}

#[test]
fn rust_generation_preserves_wrapped_values_and_ignores_legacy_name_inference() {
    let mut catalog = family(["unrelated", "another_name"], "ByTerm<Sats>", Index::Height);
    let TreeNode::Branch(branch) = &mut catalog else {
        unreachable!()
    };
    // Deliberately violate the old naming contract. Rust uses exact bindings.
    branch.field_suffixes = true;
    let directory = tempdir().unwrap();
    let path = directory.path().join("client.rs");
    generate_clients(
        &catalog,
        r#"{"openapi":"3.1.0","info":{"title":"Fixture","version":"1"},"paths":{}}"#,
        &ClientOutputPaths::new().rust(&path),
    )
    .unwrap();
    let generated = fs::read_to_string(path).unwrap();
    assert!(generated.contains("pub struct CatalogStats<T0>"));
    assert!(generated.contains("SeriesPattern1<ByTerm<Sats>>"));
    assert!(generated.contains("CatalogBinding::Leaf(\"unrelated\")"));
    assert!(generated.contains("CatalogBinding::Leaf(\"another_name\")"));
}
