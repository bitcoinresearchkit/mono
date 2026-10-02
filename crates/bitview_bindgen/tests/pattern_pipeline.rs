use std::{collections::BTreeSet, process::Command};

use bitview_bindgen::{ClientMetadata, generate_javascript_client};
use bitview_catalog::{SeriesLeaf, SeriesLeafWithSchema, TreeNode};
use brk_types::Index;
use serde_json::json;
use tempfile::tempdir;

fn leaf(name: String, kind: &str, variant: usize) -> TreeNode {
    let indexes = if variant.is_multiple_of(3) {
        BTreeSet::from([Index::Height, Index::Day1])
    } else {
        BTreeSet::from([Index::Height])
    };
    TreeNode::Leaf(SeriesLeafWithSchema::new(
        SeriesLeaf::new(name, kind.to_owned(), indexes),
        json!({"type": "integer"}),
    ))
}

#[test]
fn ancestor_factories_preserve_descendant_outlier_names() {
    let catalog = TreeNode::branch(
        ["cointime", "coinflow"]
            .into_iter()
            .map(|owner| {
                let cohorts = ["all", "sth", "lth"]
                    .into_iter()
                    .map(|cohort| {
                        let cost = if cohort == "all" {
                            format!("{owner}_cost_basis")
                        } else {
                            format!("{cohort}_{owner}_cost_basis")
                        };
                        let metrics = [
                            (
                                "capitalized_price",
                                format!("{owner}_urpd_{cohort}_capitalized_price"),
                            ),
                            ("cost_basis", cost),
                            (
                                "supply_density",
                                format!("{owner}_urpd_{cohort}_supply_density"),
                            ),
                        ]
                        .into_iter()
                        .map(|(field, name)| (field.into(), leaf(name, "Cents", 0)))
                        .collect();
                        (cohort.into(), TreeNode::branch(metrics))
                    })
                    .collect();
                (owner.into(), TreeNode::branch(cohorts))
            })
            .collect(),
    );
    let metadata = ClientMetadata::from_catalog(catalog);
    let directory = tempdir().unwrap();
    let path = directory.path().join("client.mjs");
    generate_javascript_client(&metadata, &[], &Default::default(), &path).unwrap();
    let script = format!(
        "import {{ BitviewClient }} from {};\nconst {{series}} = new BitviewClient('http://fixture.invalid');\nfor (const owner of ['cointime', 'coinflow']) {{\nfor (const cohort of ['all', 'sth', 'lth']) {{\nconst metrics = series[owner][cohort];\nconst prefix = cohort === 'all' ? '' : cohort + '_';\nif (metrics.costBasis.by.height.path !== '/api/series/' + prefix + owner + '_cost_basis/height') throw Error('Incorrect cost basis name');\nif (metrics.capitalizedPrice.by.height.path !== '/api/series/' + owner + '_urpd_' + cohort + '_capitalized_price/height') throw Error('Incorrect capitalized price name');\n}}\n}}",
        serde_json::to_string(&path.to_string_lossy()).unwrap(),
    );
    let result = Command::new("node")
        .args(["--input-type=module", "-e", &script])
        .output()
        .unwrap();
    assert!(
        result.status.success(),
        "{}",
        String::from_utf8_lossy(&result.stderr)
    );
}
