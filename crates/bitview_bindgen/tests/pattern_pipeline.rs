use std::{collections::BTreeSet, fs, process::Command};

use bitview_bindgen::{
    ClientMetadata, Endpoint, Parameter, extract_endpoints, generate_javascript_client,
    generate_python_client, generate_rust_client, parse_openapi_json,
};
use bitview_catalog::{SeriesLeaf, SeriesLeafWithSchema, TreeNode};
use brk_types::Index;
use indexmap::IndexMap;
use oas3::Spec;
use serde_json::json;
use tempfile::tempdir;

fn spec() -> Spec {
    parse_openapi_json(r#"{
        "openapi":"3.1.0", "info":{"title":"Fixture","version":"1"},
        "paths":{
            "/api/values/{height}":{"get":{
                "parameters":[
                    {"name":"height","in":"path","required":true,"schema":{"type":"integer"}},
                    {"name":"limit","in":"query","required":false,"schema":{"type":"integer"}},
                    {"name":"from","in":"query","required":true,"schema":{"type":"string"}},
                    {"name":"ids[]","in":"query","required":false,"schema":{"type":"array","items":{"type":"string"}}}
                ],
                "responses":{"200":{"description":"Success","content":{"application/json":{"schema":{"type":"integer"}}}}}
            }},
            "/api/values":{"post":{
                "operationId":"post_values",
                "requestBody":{"required":true,"content":{"application/json":{"schema":{"type":"array","items":{"type":"integer"}}}}},
                "responses":{"200":{"description":"Success","content":{"text/plain":{"schema":{"type":"string"}}}}}
            }},
            "/api/block-hash/{hash}":{"get":{
                "parameters":[{"name":"hash","in":"path","required":true,"schema":{"type":"string"}}],
                "responses":{"200":{"description":"Success","content":{"application/octet-stream":{"schema":{"type":"string","format":"binary"}}}}}
            }}
        }
    }"#).unwrap()
}

fn endpoints() -> Vec<Endpoint> {
    extract_endpoints(&spec())
}

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

fn catalog(variant: usize, width: usize) -> TreeNode {
    let mut root = IndexMap::new();
    for i in 0..width {
        let mut children = IndexMap::new();
        for (j, field) in ["close", "high", "low", "open"].into_iter().enumerate() {
            let name = match variant % 4 {
                0 => format!("cohort_{i}_{field}"),
                1 => format!("{field}_cohort_{i}"),
                2 if j == 3 => format!("unrelated_{i}"),
                _ => format!("cohort_{i}_{field}_usd"),
            };
            let kind = match (variant / 4) % 4 {
                0 => {
                    if i % 2 == 0 {
                        "Cents"
                    } else {
                        "Sats"
                    }
                }
                1 => ["Close<Cents>", "High<Cents>", "Low<Cents>", "Open<Cents>"][j],
                2 => {
                    if j % 2 == 0 {
                        "StoredU32"
                    } else {
                        "StoredU64"
                    }
                }
                _ => "Sats",
            };
            children.insert(field.to_owned(), leaf(name, kind, variant));
        }
        if variant % 2 == 1 {
            children.reverse();
        }
        root.insert(format!("cohort_{i}"), TreeNode::branch(children));
    }
    if variant >= 16 {
        root.insert("empty".into(), TreeNode::branch(IndexMap::new()));
        root.insert("nested".into(), catalog(variant - 16, 3));
    }
    TreeNode::branch(root)
}

#[test]
fn repeated_generic_mixed_and_nested_catalogs_generate_deterministically() {
    let endpoints = endpoints();
    for variant in 0..24 {
        let catalog = catalog(variant, 6);
        let first = ClientMetadata::from_catalog(catalog.clone());
        let second = ClientMetadata::from_catalog(catalog);
        assert_eq!(format!("{first:?}"), format!("{second:?}"));
        let dir = tempdir().unwrap();
        let schemas = Default::default();
        for (metadata, prefix) in [(&first, "first"), (&second, "second")] {
            generate_javascript_client(
                metadata,
                &endpoints,
                &schemas,
                &dir.path().join(format!("{prefix}.js")),
            )
            .unwrap();
            generate_python_client(
                metadata,
                &endpoints,
                &schemas,
                &dir.path().join(format!("{prefix}.py")),
            )
            .unwrap();
        }
        for extension in ["js", "py"] {
            assert_eq!(
                fs::read(dir.path().join(format!("first.{extension}"))).unwrap(),
                fs::read(dir.path().join(format!("second.{extension}"))).unwrap()
            );
        }
        let python = fs::read_to_string(dir.path().join("first.py")).unwrap();
        assert!(python.ends_with('\n') && !python.ends_with("\n\n"));
        assert!(python.lines().all(|line| line == line.trim_end()));
    }
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

/// Unicode series bases must survive metadata analysis and client emission.
#[test]
fn unicode_catalog_names_reach_all_three_generated_clients() {
    fn prefix_names(node: &mut TreeNode) {
        match node {
            TreeNode::Leaf(leaf) => leaf.leaf.name.insert_str(0, "é_"),
            TreeNode::Branch(children) => children.values_mut().for_each(prefix_names),
        }
    }
    let mut tree = catalog(0, 3);
    prefix_names(&mut tree);
    let metadata = ClientMetadata::from_catalog(tree);
    assert!(
        metadata
            .get_node_base("cohort_0")
            .unwrap()
            .base
            .starts_with("é_")
    );
    let dir = tempdir().unwrap();
    generate_javascript_client(
        &metadata,
        &[],
        &Default::default(),
        &dir.path().join("client.js"),
    )
    .unwrap();
    generate_python_client(
        &metadata,
        &[],
        &Default::default(),
        &dir.path().join("client.py"),
    )
    .unwrap();
    generate_rust_client(&metadata.catalog, &[], &dir.path().join("client.rs")).unwrap();
    for name in ["client.js", "client.py", "client.rs"] {
        assert!(
            fs::read_to_string(dir.path().join(name))
                .unwrap()
                .contains("é_cohort")
        );
    }
}

#[test]
fn python_optional_body_and_csv_keep_parameter_order_and_returns() {
    let mut endpoints = endpoints();
    let get = endpoints
        .iter_mut()
        .find(|endpoint| endpoint.path == "/api/values/{height}")
        .unwrap();
    get.supports_csv = true;
    get.query_params.push(Parameter {
        name: "format".into(),
        param_type: "string".into(),
        required: false,
        description: None,
        schema: json!({"type":"string"}),
    });
    endpoints
        .iter_mut()
        .find(|endpoint| endpoint.method == "POST")
        .unwrap()
        .request_body
        .as_mut()
        .unwrap()
        .required = false;
    let metadata = ClientMetadata::from_catalog(catalog(0, 2));
    let dir = tempdir().unwrap();
    let path = dir.path().join("client.py");
    generate_python_client(&metadata, &endpoints, &Default::default(), &path).unwrap();
    let output = fs::read_to_string(path).unwrap();
    let signature = output
        .lines()
        .find(|line| line.contains("def get_values_by_height("))
        .unwrap();
    assert!(signature.find("from_: str").unwrap() < signature.find("limit: Optional").unwrap());
    assert_eq!(output.matches("if format == 'csv':").count(), 1);
    assert!(output.contains("body: Optional[List[int]] = None"));
}
