use std::{collections::BTreeSet, fs};

use bitview_bindgen::{ClientOutputPaths, generate_clients};
use bitview_catalog::{SeriesLeaf, SeriesLeafWithSchema, TreeNode};
use brk_types::Index;
use indexmap::IndexMap;
use serde_json::json;
use tempfile::tempdir;

fn catalog(declared: bool) -> TreeNode {
    TreeNode::branch(
        ["amount", "value"]
            .into_iter()
            .enumerate()
            .map(|(i, base)| {
                let children = ["min", "max", "pct10", "pct25", "median", "pct75", "pct90"]
                    .into_iter()
                    .map(|field| {
                        (
                            field.to_string(),
                            TreeNode::Leaf(SeriesLeafWithSchema::new(
                                SeriesLeaf::new(
                                    format!("{base}_{field}"),
                                    if i == 0 { "Sats" } else { "Cents" }.into(),
                                    BTreeSet::from([Index::Height, Index::Day1]),
                                ),
                                json!({"type": "integer"}),
                            )),
                        )
                    })
                    .collect::<IndexMap<_, _>>();
                let mut family = TreeNode::branch(children);
                if declared {
                    family = family.with_field_suffixes();
                }
                (base.to_string(), family)
            })
            .collect(),
    )
}

#[test]
fn declared_family_and_legacy_inference_emit_identical_outputs() {
    let directory = tempdir().unwrap();
    for declared in [false, true] {
        let root = directory.path().join(declared.to_string());
        let paths = ClientOutputPaths::new()
            .rust(root.join("client.rs"))
            .cli(root.join("cli.rs"))
            .javascript(root.join("client.js"))
            .python(root.join("client.py"))
            .llm(root.join("llm"))
            .llm_manifest(root.join("manifest.json"));
        generate_clients(
            &catalog(declared),
            r#"{"openapi":"3.1.0","info":{"title":"Fixture","version":"1"},"paths":{}}"#,
            &paths,
        )
        .unwrap();
    }
    for file in [
        "client.rs",
        "cli.rs",
        "client.js",
        "client.py",
        "llm/llms.txt",
        "llm/llms-full.txt",
        "manifest.json",
    ] {
        assert_eq!(
            fs::read(directory.path().join("false").join(file)).unwrap(),
            fs::read(directory.path().join("true").join(file)).unwrap(),
            "{file}"
        );
    }
}
