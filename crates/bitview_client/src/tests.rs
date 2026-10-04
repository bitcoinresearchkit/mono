use std::{collections::BTreeMap, fs, path::Path};

use crate::{AnySeriesPattern, BitviewClient, generated::Node};

/// Every typed path of the API contract resolves to its series and indexes, and the tree holds
/// nothing else. The contract is a workspace snapshot: a published copy of this crate (packaged
/// with `.cargo_vcs_info.json`) has nothing to compare against.
#[test]
fn typed_paths_resolve() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    if manifest.join(".cargo_vcs_info.json").exists() {
        return;
    }
    let snapshot = manifest.join("../bitview_devtools/snapshots/client-paths.tsv");
    let snapshot =
        fs::read_to_string(snapshot).expect("run `cargo api` to record client-paths.tsv");
    let sorted = |indexes: Vec<&str>| {
        let mut indexes: Vec<String> = indexes.into_iter().map(str::to_owned).collect();
        indexes.sort();
        indexes
    };
    let mut expected = BTreeMap::new();
    for line in snapshot
        .lines()
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
    {
        let columns: Vec<&str> = line.split('\t').collect();
        assert_eq!(columns.len(), 5, "malformed client-paths.tsv row: {line:?}");
        let indexes = sorted(columns[4].split(',').filter(|i| !i.is_empty()).collect());
        let repeated = expected.insert(columns[1].to_owned(), (columns[0].to_owned(), indexes));
        assert!(
            repeated.is_none(),
            "client-paths.tsv repeats {}",
            columns[1]
        );
    }

    let client = BitviewClient::new("http://fixture.invalid");
    let mut actual = BTreeMap::new();
    client
        .series()
        .visit("", &mut |path, leaf: &dyn AnySeriesPattern| {
            let indexes = sorted(leaf.indexes().iter().map(|index| index.name()).collect());
            actual.insert(path.to_owned(), (leaf.name().to_owned(), indexes));
        });

    let wrong: Vec<String> = expected
        .iter()
        .filter(|(path, leaf)| actual.get(*path) != Some(leaf))
        .map(|(path, leaf)| format!("{path}: expected {leaf:?}, got {:?}", actual.get(path)))
        .collect();
    let extra: Vec<String> = actual
        .keys()
        .filter(|path| !expected.contains_key(*path))
        .map(|path| format!("{path}: not in client-paths.tsv"))
        .collect();
    assert!(
        wrong.is_empty() && extra.is_empty(),
        "{} wrong or missing, {} extra:\n{}",
        wrong.len(),
        extra.len(),
        wrong
            .iter()
            .take(20)
            .chain(extra.iter().take(20))
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );
}
