use std::collections::BTreeMap;

use brk_traversable::{IndexMap, Traversable, TreeNode};
use vecdb::AnyExportableVec;

struct TestLeaf(&'static str);

impl Traversable for TestLeaf {
    fn to_tree_node(&self) -> TreeNode {
        TreeNode::Branch(IndexMap::new())
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        std::iter::empty()
    }

    fn collect_series_descriptions<'a>(
        &'a self,
        description_fragments: &mut Vec<&'static str>,
        descriptions: &mut BTreeMap<&'a str, Vec<&'static str>>,
    ) {
        descriptions.insert(self.0, description_fragments.clone());
    }
}

#[derive(brk_traversable_derive::Traversable)]
struct TestTree {
    /// Reported in the child representation.
    #[traversable(rename = "metric")]
    pub represented: TestLeaf,
    pub inherited: TestLeaf,
}

#[derive(brk_traversable_derive::Traversable)]
struct TestRoot {
    /// The exact metric definition. Continues on a second line.
    pub metrics: TestTree,
}

#[test]
fn joins_every_documented_field_on_the_path_in_order() {
    let tree = TestRoot {
        metrics: TestTree {
            represented: TestLeaf("represented"),
            inherited: TestLeaf("inherited"),
        },
    };
    let mut description_fragments = Vec::new();
    let mut descriptions = BTreeMap::new();

    tree.collect_series_descriptions(&mut description_fragments, &mut descriptions);

    assert_eq!(
        descriptions.get("represented").unwrap(),
        &[
            "The exact metric definition. Continues on a second line.",
            "Reported in the child representation.",
        ]
    );
    assert_eq!(
        descriptions.get("inherited").unwrap(),
        &["The exact metric definition. Continues on a second line."]
    );
    assert!(description_fragments.is_empty());
}
