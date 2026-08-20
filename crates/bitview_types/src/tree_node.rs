use std::{collections::BTreeSet, mem};

use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{SeriesLeaf, SeriesLeafWithSchema};

/// Hierarchical tree node for organizing series into categories
#[derive(Debug, Clone, Serialize, PartialEq, Eq, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum TreeNode {
    /// Branch node containing subcategories
    Branch(IndexMap<String, TreeNode>),
    /// Leaf node containing series metadata with schema
    Leaf(SeriesLeafWithSchema),
}

const BASE: &str = "raw";

impl TreeNode {
    pub fn is_empty(&self) -> bool {
        if let Self::Branch(tree) = self {
            tree.is_empty()
        } else {
            false
        }
    }

    pub fn as_mut_branch(&mut self) -> &mut IndexMap<String, TreeNode> {
        match self {
            Self::Branch(b) => b,
            _ => panic!(),
        }
    }

    /// Wraps a node in a Branch with the given key.
    /// Used by #[traversable(wrap = "...")] to produce Branch { key: inner }.
    pub fn wrap(key: &str, inner: Self) -> Self {
        let mut map = IndexMap::new();
        map.insert(key.to_string(), inner);
        Self::Branch(map)
    }

    /// Merges all first-level branches into a single flattened structure (consuming version).
    /// Direct leaves use their key (use #[traversable(rename = "...")] to control).
    /// Branch children are lifted with their keys.
    /// If all resulting children are leaves with the same series name, collapses to a single leaf.
    /// Returns None if conflicts are found (same key with incompatible values).
    pub fn merge_branches(self) -> Option<Self> {
        let Self::Branch(tree) = self else {
            return Some(self);
        };

        let mut merged: IndexMap<String, TreeNode> = IndexMap::new();

        for (key, node) in tree {
            match node {
                Self::Leaf(leaf) => {
                    // Direct leaves use their key (which may be renamed via attribute)
                    Self::merge_node(&mut merged, key, Self::Leaf(leaf))?;
                }
                Self::Branch(inner) => {
                    // Lift children from branches with their keys
                    for (inner_key, inner_node) in inner {
                        Self::merge_node(&mut merged, inner_key, inner_node)?;
                    }
                }
            }
        }

        // If all children are leaves with the same series name, collapse into single leaf
        Some(Self::try_collapse_same_name_leaves(merged))
    }

    /// If all entries in the map are leaves with the same series name,
    /// collapse them into a single leaf with merged indexes.
    fn try_collapse_same_name_leaves(map: IndexMap<String, TreeNode>) -> Self {
        if map.is_empty() {
            return Self::Branch(map);
        }

        // Check if all entries are leaves with the same name
        let mut first_leaf: Option<&SeriesLeafWithSchema> = None;
        let mut merged_indexes = BTreeSet::new();

        for node in map.values() {
            match node {
                Self::Leaf(leaf) => {
                    if let Some(first) = &first_leaf {
                        if leaf.name() != first.name() {
                            // Different names - can't collapse
                            return Self::Branch(map);
                        }
                    } else {
                        first_leaf = Some(leaf);
                    }
                    merged_indexes.extend(leaf.indexes().iter().copied());
                }
                Self::Branch(_) => {
                    // Has non-leaf entries - can't collapse
                    return Self::Branch(map);
                }
            }
        }

        // All entries were leaves with the same name
        let first = first_leaf.unwrap();
        Self::Leaf(SeriesLeafWithSchema::new(
            SeriesLeaf::new(
                first.name().to_string(),
                first.kind().to_string(),
                merged_indexes,
            ),
            first.schema.clone(),
        ))
    }

    /// Merges a node into the target map at the given key (consuming version).
    /// Returns None if there's a conflict.
    pub fn merge_node(
        target: &mut IndexMap<String, TreeNode>,
        key: String,
        node: TreeNode,
    ) -> Option<()> {
        match target.get_mut(&key) {
            None => {
                target.insert(key, node);
                Some(())
            }
            Some(existing) => {
                match (existing, node) {
                    (Self::Leaf(a), Self::Leaf(b)) if a.is_same_series(&b) => {
                        a.merge_indexes(&b);
                        Some(())
                    }
                    (Self::Leaf(a), Self::Leaf(b)) => {
                        eprintln!("Conflict: Different leaf values for key '{key}'");
                        eprintln!("  Existing: {a:?}");
                        eprintln!("  New: {b:?}");
                        None
                    }
                    (existing @ Self::Leaf(_), Self::Branch(branch)) => {
                        let Self::Leaf(leaf) =
                            mem::replace(existing, Self::Branch(IndexMap::new()))
                        else {
                            unreachable!()
                        };
                        let Self::Branch(new_branch) = existing else {
                            unreachable!()
                        };
                        new_branch.insert(BASE.to_string(), Self::Leaf(leaf));

                        for (k, v) in branch {
                            Self::merge_node(new_branch, k, v)?;
                        }
                        Some(())
                    }
                    (Self::Branch(existing_branch), Self::Leaf(leaf)) => {
                        Self::merge_node(existing_branch, BASE.to_string(), Self::Leaf(leaf))?;
                        Some(())
                    }
                    // Both branches: merge recursively
                    (Self::Branch(existing_branch), Self::Branch(new_inner)) => {
                        for (k, v) in new_inner {
                            Self::merge_node(existing_branch, k, v)?;
                        }
                        Some(())
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use brk_types::Index;
    use serde_json::Value;

    fn leaf(name: &str, index: Index) -> TreeNode {
        TreeNode::Leaf(SeriesLeafWithSchema {
            leaf: SeriesLeaf {
                name: name.to_string(),
                kind: "TestType".to_string(),
                indexes: BTreeSet::from([index]),
            },
            openapi_type: "object".to_string(),
            schema: Value::Null,
        })
    }

    fn branch(children: Vec<(&str, TreeNode)>) -> TreeNode {
        TreeNode::Branch(
            children
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect(),
        )
    }

    fn get_leaf_indexes(node: &TreeNode) -> Option<&BTreeSet<Index>> {
        match node {
            TreeNode::Leaf(l) => Some(&l.leaf.indexes),
            _ => None,
        }
    }

    // ========== Leaf passthrough ==========

    #[test]
    fn merge_leaf_passthrough() {
        let tree = leaf("s", Index::Height);
        let merged = tree.merge_branches().unwrap();
        assert!(matches!(merged, TreeNode::Leaf(_)));
    }

    #[test]
    fn merge_empty_branch() {
        let tree = branch(vec![]);
        let merged = tree.merge_branches().unwrap();
        match merged {
            TreeNode::Branch(map) => assert!(map.is_empty()),
            _ => panic!("Expected empty branch"),
        }
    }

    // ========== Direct leaves keep their keys ==========

    #[test]
    fn merge_direct_leaves_keep_keys() {
        // Direct leaves with different keys stay separate
        let tree = branch(vec![
            ("sum", leaf("s_sum", Index::Height)),
            ("cumulative", leaf("s_cumulative", Index::Height)),
        ]);
        let merged = tree.merge_branches().unwrap();

        match merged {
            TreeNode::Branch(map) => {
                assert_eq!(map.len(), 2);
                assert!(map.contains_key("sum"));
                assert!(map.contains_key("cumulative"));
            }
            _ => panic!("Expected branch"),
        }
    }

    // ========== Branch lifting ==========

    #[test]
    fn merge_lifts_branch_children() {
        // Branch children are lifted to top level with their keys
        let tree = branch(vec![(
            "week1",
            branch(vec![
                ("sum", leaf("s_sum", Index::Week1)),
                ("cumulative", leaf("s_cumulative", Index::Week1)),
            ]),
        )]);
        let merged = tree.merge_branches().unwrap();

        match merged {
            TreeNode::Branch(map) => {
                assert!(map.contains_key("sum"));
                assert!(map.contains_key("cumulative"));
                assert!(!map.contains_key("week1")); // Parent key gone
            }
            _ => panic!("Expected branch"),
        }
    }

    #[test]
    fn merge_multiple_branches_combines_indexes() {
        // Multiple branches with same child keys → indexes are merged
        let tree = branch(vec![
            (
                "week1",
                branch(vec![
                    ("sum", leaf("s_sum", Index::Week1)),
                    ("cumulative", leaf("s_cumulative", Index::Week1)),
                ]),
            ),
            (
                "month1",
                branch(vec![
                    ("sum", leaf("s_sum", Index::Month1)),
                    ("cumulative", leaf("s_cumulative", Index::Month1)),
                ]),
            ),
        ]);
        let merged = tree.merge_branches().unwrap();

        match merged {
            TreeNode::Branch(map) => {
                let sum_indexes = get_leaf_indexes(map.get("sum").unwrap()).unwrap();
                assert!(sum_indexes.contains(&Index::Week1));
                assert!(sum_indexes.contains(&Index::Month1));

                let cumulative_indexes = get_leaf_indexes(map.get("cumulative").unwrap()).unwrap();
                assert!(cumulative_indexes.contains(&Index::Week1));
                assert!(cumulative_indexes.contains(&Index::Month1));
            }
            _ => panic!("Expected branch"),
        }
    }

    // ========== Mixed leaves and branches ==========

    #[test]
    fn merge_leaf_merges_with_lifted_branch_child() {
        // Direct leaf with key "cumulative" merges with lifted "cumulative" from branch
        // This simulates: height_cumulative (renamed) + day1 branch
        let tree = branch(vec![
            ("cumulative", leaf("s_cumulative", Index::Height)),
            (
                "day1",
                branch(vec![
                    ("sum", leaf("s_sum", Index::Day1)),
                    ("cumulative", leaf("s_cumulative", Index::Day1)),
                ]),
            ),
        ]);
        let merged = tree.merge_branches().unwrap();

        match merged {
            TreeNode::Branch(map) => {
                assert_eq!(map.len(), 2);

                // cumulative merged: Height + Day1
                let cumulative_indexes = get_leaf_indexes(map.get("cumulative").unwrap()).unwrap();
                assert!(cumulative_indexes.contains(&Index::Height));
                assert!(cumulative_indexes.contains(&Index::Day1));

                // sum only has Day1
                let sum_indexes = get_leaf_indexes(map.get("sum").unwrap()).unwrap();
                assert!(sum_indexes.contains(&Index::Day1));
                assert!(!sum_indexes.contains(&Index::Height));
            }
            _ => panic!("Expected branch"),
        }
    }

    #[test]
    fn merge_derived_computed_block_sum_cum_pattern() {
        // Simulates DerivedComputedBlockSumCum:
        // - height_cumulative (renamed to "cumulative") → direct leaf at Height
        // - day1 → branch with sum/cumulative at Day1
        // - week1 (flattened from dates) → branch with sum/cumulative at Week1
        // - epoch → branch with sum/cumulative at Epoch
        let tree = branch(vec![
            ("cumulative", leaf("s_cumulative", Index::Height)),
            (
                "day1",
                branch(vec![
                    ("sum", leaf("s_sum", Index::Day1)),
                    ("cumulative", leaf("s_cumulative", Index::Day1)),
                ]),
            ),
            (
                "week1",
                branch(vec![
                    ("sum", leaf("s_sum", Index::Week1)),
                    ("cumulative", leaf("s_cumulative", Index::Week1)),
                ]),
            ),
            (
                "epoch",
                branch(vec![
                    ("sum", leaf("s_sum", Index::Epoch)),
                    ("cumulative", leaf("s_cumulative", Index::Epoch)),
                ]),
            ),
        ]);

        let merged = tree.merge_branches().unwrap();

        match merged {
            TreeNode::Branch(map) => {
                assert_eq!(map.len(), 2);

                // sum: Day1, Week1, Epoch (NOT Height)
                let sum_indexes = get_leaf_indexes(map.get("sum").unwrap()).unwrap();
                assert!(!sum_indexes.contains(&Index::Height));
                assert!(sum_indexes.contains(&Index::Day1));
                assert!(sum_indexes.contains(&Index::Week1));
                assert!(sum_indexes.contains(&Index::Epoch));

                // cumulative: Height, Day1, Week1, Epoch
                let cumulative_indexes = get_leaf_indexes(map.get("cumulative").unwrap()).unwrap();
                assert!(cumulative_indexes.contains(&Index::Height));
                assert!(cumulative_indexes.contains(&Index::Day1));
                assert!(cumulative_indexes.contains(&Index::Week1));
                assert!(cumulative_indexes.contains(&Index::Epoch));
            }
            _ => panic!("Expected branch"),
        }
    }

    // ========== Conflict detection ==========

    #[test]
    fn merge_conflict_from_lifted_branches() {
        // Two branches lifting children with same key but different series names → conflict
        let tree = branch(vec![
            ("a", branch(vec![("data", leaf("s_a", Index::Height))])),
            ("b", branch(vec![("data", leaf("s_b", Index::Day1))])),
        ]);
        let result = tree.merge_branches();
        assert!(result.is_none(), "Should detect conflict");
    }

    #[test]
    fn merge_no_conflict_same_series_different_indexes() {
        // Same key, same series name, different indexes → merges indexes → collapses to Leaf
        let tree = branch(vec![
            ("a", branch(vec![("sum", leaf("s_sum", Index::Height))])),
            ("b", branch(vec![("sum", leaf("s_sum", Index::Day1))])),
        ]);
        let result = tree.merge_branches();
        assert!(result.is_some(), "Should merge successfully");

        let merged = result.unwrap();
        match merged {
            TreeNode::Leaf(leaf) => {
                assert_eq!(leaf.name(), "s_sum");
                let indexes = leaf.indexes();
                assert!(indexes.contains(&Index::Height));
                assert!(indexes.contains(&Index::Day1));
            }
            _ => panic!("Expected collapsed Leaf"),
        }
    }

    // ========== Nested branches ==========

    #[test]
    fn merge_nested_branches_flattens_one_level() {
        // Merge only flattens one level - nested branches stay as branches
        let tree = branch(vec![(
            "outer",
            branch(vec![(
                "inner",
                branch(vec![("leaf", leaf("s", Index::Height))]),
            )]),
        )]);
        let merged = tree.merge_branches().unwrap();

        // "inner" is lifted to top level but stays as a branch
        match merged {
            TreeNode::Branch(map) => {
                assert!(map.contains_key("inner"));
                assert!(matches!(map.get("inner"), Some(TreeNode::Branch(_))));
            }
            _ => panic!("Expected branch"),
        }
    }

    // ========== Collapse same-name leaves ==========

    #[test]
    fn collapse_direct_leaf_with_lifted_branches_same_name() {
        // ComputedVecsDateLast pattern:
        // - day1: direct leaf (field name as key)
        // - rest (flattened): DerivedDateLast → branches with "last" children
        // All leaves have same series name → collapse to single Leaf
        let tree = branch(vec![
            // Direct leaf from day1 field (no wrap attribute)
            ("day1", leaf("1m_block_count", Index::Day1)),
            // Flattened from rest: DerivedDateLast
            (
                "week1",
                branch(vec![("last", leaf("1m_block_count", Index::Week1))]),
            ),
            (
                "month1",
                branch(vec![("last", leaf("1m_block_count", Index::Month1))]),
            ),
        ]);

        let merged = tree.merge_branches().unwrap();

        // All leaves have same name "1m_block_count" → collapses to single Leaf
        match &merged {
            TreeNode::Leaf(leaf) => {
                assert_eq!(leaf.name(), "1m_block_count");
                let indexes = leaf.indexes();
                assert!(indexes.contains(&Index::Day1));
                assert!(indexes.contains(&Index::Week1));
                assert!(indexes.contains(&Index::Month1));
            }
            TreeNode::Branch(map) => {
                panic!(
                    "Expected collapsed leaf, got branch: {:?}",
                    map.keys().collect::<Vec<_>>()
                );
            }
        }
    }

    // ========== Case 1: DerivedDateLast (all same series name) ==========

    #[test]
    fn case1_derived_date_last() {
        // All leaves have the same series name, all wrapped as "last"
        // All branches lift to same key → collapses to single Leaf
        let tree = branch(vec![
            (
                "week1",
                branch(vec![("last", leaf("1m_block_count", Index::Week1))]),
            ),
            (
                "month1",
                branch(vec![("last", leaf("1m_block_count", Index::Month1))]),
            ),
            (
                "month3",
                branch(vec![("last", leaf("1m_block_count", Index::Month3))]),
            ),
            (
                "year1",
                branch(vec![("last", leaf("1m_block_count", Index::Year1))]),
            ),
        ]);

        let merged = tree.merge_branches().unwrap();

        match &merged {
            TreeNode::Leaf(leaf) => {
                assert_eq!(leaf.name(), "1m_block_count");
                let indexes = leaf.indexes();
                assert!(indexes.contains(&Index::Week1));
                assert!(indexes.contains(&Index::Month1));
                assert!(indexes.contains(&Index::Month3));
                assert!(indexes.contains(&Index::Year1));
            }
            _ => panic!("Expected collapsed Leaf"),
        }
    }

    // ========== Case 2: SumCum (different aggregations via wrap) ==========

    #[test]
    fn case2_sum_cum() {
        // SumVec/CumulativeVec use wrap to produce branches
        // Multiple time periods, each producing { "sum": Leaf, "cumulative": Leaf }
        // These should merge into { "sum": Leaf(all indexes), "cumulative": Leaf(all indexes) }
        let tree = branch(vec![
            (
                "day1",
                branch(vec![
                    ("sum", leaf("s_sum", Index::Day1)),
                    ("cumulative", leaf("s_cumulative", Index::Day1)),
                ]),
            ),
            (
                "week1",
                branch(vec![
                    ("sum", leaf("s_sum", Index::Week1)),
                    ("cumulative", leaf("s_cumulative", Index::Week1)),
                ]),
            ),
        ]);

        let merged = tree.merge_branches().unwrap();

        // DESIRED: { "sum": Leaf, "cumulative": Leaf } with merged indexes
        match merged {
            TreeNode::Branch(map) => {
                assert_eq!(map.len(), 2);

                let sum_indexes = get_leaf_indexes(map.get("sum").unwrap()).unwrap();
                assert!(sum_indexes.contains(&Index::Day1));
                assert!(sum_indexes.contains(&Index::Week1));

                let cumulative_indexes = get_leaf_indexes(map.get("cumulative").unwrap()).unwrap();
                assert!(cumulative_indexes.contains(&Index::Day1));
                assert!(cumulative_indexes.contains(&Index::Week1));
            }
            _ => panic!("Expected branch with sum and cumulative"),
        }
    }

    // ========== Case 3: ComputedBlockSum (base + sum) ==========

    #[test]
    fn case3_computed_block_sum() {
        // ComputedBlockSum:
        // - height: wrap="raw" → Branch { "raw": Leaf(height) }
        // - rest (flatten): DerivedComputedBlockSum → branches with "sum" children
        let tree = branch(vec![
            // height wrapped as "raw"
            ("height", branch(vec![("raw", leaf("s", Index::Height))])),
            // rest (flattened) produces branches
            ("day1", branch(vec![("sum", leaf("s_sum", Index::Day1))])),
            ("week1", branch(vec![("sum", leaf("s_sum", Index::Week1))])),
        ]);

        let merged = tree.merge_branches().unwrap();

        // DESIRED: { "raw": Leaf(height), "sum": Leaf(day1, week1) }
        match &merged {
            TreeNode::Branch(map) => {
                assert_eq!(
                    map.len(),
                    2,
                    "Expected 2 keys 'base' and 'sum', got: {:?}",
                    map.keys().collect::<Vec<_>>()
                );

                // base should have Height only
                let base_indexes = get_leaf_indexes(map.get("raw").unwrap()).unwrap();
                assert!(base_indexes.contains(&Index::Height));
                assert_eq!(base_indexes.len(), 1);

                // sum should have Day1 and Week1
                let sum_indexes = get_leaf_indexes(map.get("sum").unwrap()).unwrap();
                assert!(!sum_indexes.contains(&Index::Height));
                assert!(sum_indexes.contains(&Index::Day1));
                assert!(sum_indexes.contains(&Index::Week1));
            }
            _ => panic!("Expected branch"),
        }
    }

    // ========== Case 4: ComputedBlockLast (base + last) ==========

    #[test]
    fn case4_computed_block_last() {
        // ComputedBlockLast:
        // - height: wrap="raw" → Branch { "raw": Leaf(height) }
        // - rest (flatten): DerivedComputedBlockLast → branches with "last" children
        let tree = branch(vec![
            // height wrapped as "raw"
            ("height", branch(vec![("raw", leaf("s", Index::Height))])),
            // rest (flattened) produces branches with "last" key
            ("day1", branch(vec![("last", leaf("s_last", Index::Day1))])),
            (
                "week1",
                branch(vec![("last", leaf("s_last", Index::Week1))]),
            ),
        ]);

        let merged = tree.merge_branches().unwrap();

        // DESIRED: { "raw": Leaf(height), "last": Leaf(day1, week1) }
        match &merged {
            TreeNode::Branch(map) => {
                assert_eq!(
                    map.len(),
                    2,
                    "Expected 2 keys 'base' and 'last', got: {:?}",
                    map.keys().collect::<Vec<_>>()
                );

                // base should have Height only
                let base_indexes = get_leaf_indexes(map.get("raw").unwrap()).unwrap();
                assert!(base_indexes.contains(&Index::Height));
                assert_eq!(base_indexes.len(), 1);

                // last should have Day1 and Week1
                let last_indexes = get_leaf_indexes(map.get("last").unwrap()).unwrap();
                assert!(!last_indexes.contains(&Index::Height));
                assert!(last_indexes.contains(&Index::Day1));
                assert!(last_indexes.contains(&Index::Week1));
            }
            _ => panic!("Expected branch"),
        }
    }

    // ========== Case 5: ComputedBlockFull (most complex) ==========

    #[test]
    fn case5_computed_block_full() {
        // ComputedBlockFull has:
        // - height: wrapped as "raw" (raw values, not aggregated)
        // - rest (flatten): DerivedComputedBlockFull {
        //     height_cumulative: CumulativeVec → Branch{"cumulative": Leaf}
        //     day1: Full → Branch{avg, min, max, sum, cumulative}
        //     dates (flatten): more aggregation branches
        //   }
        let tree = branch(vec![
            // height wrapped as "raw" (raw values at height granularity)
            ("height", branch(vec![("raw", leaf("s", Index::Height))])),
            // height_cumulative wrapped as cumulative
            (
                "height_cumulative",
                branch(vec![("cumulative", leaf("s_cumulative", Index::Height))]),
            ),
            // day1 Full
            (
                "day1",
                branch(vec![
                    ("average", leaf("s_average", Index::Day1)),
                    ("min", leaf("s_min", Index::Day1)),
                    ("max", leaf("s_max", Index::Day1)),
                    ("sum", leaf("s_sum", Index::Day1)),
                    ("cumulative", leaf("s_cumulative", Index::Day1)),
                ]),
            ),
            // week1 (from flattened dates)
            (
                "week1",
                branch(vec![
                    ("average", leaf("s_average", Index::Week1)),
                    ("min", leaf("s_min", Index::Week1)),
                    ("max", leaf("s_max", Index::Week1)),
                    ("sum", leaf("s_sum", Index::Week1)),
                    ("cumulative", leaf("s_cumulative", Index::Week1)),
                ]),
            ),
        ]);

        let merged = tree.merge_branches().unwrap();

        // DESIRED: { base, average, min, max, sum, cumulative } each with merged indexes
        match &merged {
            TreeNode::Branch(map) => {
                assert_eq!(
                    map.len(),
                    6,
                    "Expected 6 keys, got: {:?}",
                    map.keys().collect::<Vec<_>>()
                );

                // base should have Height only
                let base_indexes = get_leaf_indexes(map.get("raw").unwrap()).unwrap();
                assert!(base_indexes.contains(&Index::Height));
                assert_eq!(base_indexes.len(), 1);

                // cumulative should include Height (from height_cumulative)
                let cumulative_indexes = get_leaf_indexes(map.get("cumulative").unwrap()).unwrap();
                assert!(
                    cumulative_indexes.contains(&Index::Height),
                    "cumulative should include Height"
                );
                assert!(cumulative_indexes.contains(&Index::Day1));
                assert!(cumulative_indexes.contains(&Index::Week1));

                // average, min, max, sum should have Day1 and Week1 only
                for key in ["average", "min", "max", "sum"] {
                    let indexes = get_leaf_indexes(map.get(key).unwrap()).unwrap();
                    assert!(!indexes.contains(&Index::Height));
                    assert!(indexes.contains(&Index::Day1));
                    assert!(indexes.contains(&Index::Week1));
                }
            }
            _ => panic!("Expected branch"),
        }
    }

    // ========== Case 6: LazyDateLast (all branches with same inner key) ==========

    #[test]
    fn case6_lazy_date_last_all_branches_same_key_collapses() {
        // LazyDateLast pattern: All fields are branches with same inner key "last"
        // All leaves have the same series name → should collapse to single Leaf
        let tree = branch(vec![
            (
                "day1",
                branch(vec![("last", leaf("price_200d_sma", Index::Day1))]),
            ),
            (
                "week1",
                branch(vec![("last", leaf("price_200d_sma", Index::Week1))]),
            ),
            (
                "month1",
                branch(vec![("last", leaf("price_200d_sma", Index::Month1))]),
            ),
            (
                "month3",
                branch(vec![("last", leaf("price_200d_sma", Index::Month3))]),
            ),
            (
                "year1",
                branch(vec![("last", leaf("price_200d_sma", Index::Year1))]),
            ),
        ]);

        let merged = tree.merge_branches().unwrap();

        // All branches lifted to same "last" key, all same series name → collapse to Leaf
        match &merged {
            TreeNode::Leaf(leaf) => {
                assert_eq!(leaf.name(), "price_200d_sma");
                let indexes = leaf.indexes();
                assert!(indexes.contains(&Index::Day1));
                assert!(indexes.contains(&Index::Week1));
                assert!(indexes.contains(&Index::Month1));
                assert!(indexes.contains(&Index::Month3));
                assert!(indexes.contains(&Index::Year1));
            }
            TreeNode::Branch(map) => {
                panic!(
                    "Expected collapsed Leaf, got Branch with keys: {:?}",
                    map.keys().collect::<Vec<_>>()
                );
            }
        }
    }

    // ========== Case 7: LazyBlockValue ==========
    // LazyBlockValue (no merge):
    //   - sats: LazyVec with wrap="sats"
    //   - rest: LazyDerivedBlockValue with flatten
    //     - bitcoin: LazyVec (plain field)
    //     - dollars: optional derived vector (plain field)

    #[test]
    fn case7_lazy_block_value() {
        // Simulates the tree produced by LazyBlockValue
        // sats wrapped, rest flattened with bitcoin/dollars as plain leaves
        let tree = branch(vec![
            // sats with wrap="sats" produces Branch { sats: Leaf }
            ("sats", branch(vec![("sats", leaf("s", Index::Height))])),
            // rest with flatten: LazyDerivedBlockValue fields lifted
            (
                "rest",
                branch(vec![
                    ("bitcoin", leaf("s_btc", Index::Height)),
                    ("dollars", leaf("s_usd", Index::Height)),
                ]),
            ),
        ]);

        // After merge_branches: lifts children, flattens rest
        let merged = tree.merge_branches().unwrap();

        match &merged {
            TreeNode::Branch(map) => {
                assert_eq!(
                    map.len(),
                    3,
                    "Expected sats, bitcoin, dollars. Got: {:?}",
                    map.keys().collect::<Vec<_>>()
                );
                assert!(matches!(map.get("sats"), Some(TreeNode::Leaf(_))));
                assert!(matches!(map.get("bitcoin"), Some(TreeNode::Leaf(_))));
                assert!(matches!(map.get("dollars"), Some(TreeNode::Leaf(_))));
            }
            _ => panic!("Expected branch"),
        }
    }

    // ========== Case 8: BinaryBlockSumCum ==========
    // After derive applies all inner merges and flatten, before parent merge:
    // - height wrapped as "raw" → { base: Leaf(Height) }
    // - height_cumulative wrapped as "cumulative" → { cumulative: Leaf(Height) }
    // - rest (flatten): children from already-merged inner struct inserted directly
    //
    // The key insight: inner types have merge applied BEFORE flatten lifts them.
    // So rest.to_tree_node() returns merged { sum, cumulative } directly.

    #[test]
    fn case8_binary_block_sum_cum() {
        // Tree AFTER derive applies inner merges, flatten lifts rest's children:
        let tree = branch(vec![
            // height with wrap="raw"
            ("height", branch(vec![("raw", leaf("s", Index::Height))])),
            // height_cumulative with wrap="cumulative"
            (
                "height_cumulative",
                branch(vec![("cumulative", leaf("s_cumulative", Index::Height))]),
            ),
            // From rest (flatten) - inner struct already merged to { sum, cumulative }
            // Each leaf has merged indexes from all time periods
            (
                "sum",
                leaf("s_sum", Index::Day1), // Would have all time indexes
            ),
            (
                "cumulative",
                leaf("s_cumulative", Index::Day1), // Would have all time indexes
            ),
        ]);

        let merged = tree.merge_branches().unwrap();

        // Expected: { base, sum, cumulative }
        match &merged {
            TreeNode::Branch(map) => {
                assert_eq!(
                    map.len(),
                    3,
                    "Expected base, sum, cumulative. Got: {:?}",
                    map.keys().collect::<Vec<_>>()
                );

                // base: only Height
                let base_indexes = get_leaf_indexes(map.get("raw").unwrap()).unwrap();
                assert_eq!(base_indexes.len(), 1);
                assert!(base_indexes.contains(&Index::Height));

                // sum: from flattened rest
                assert!(matches!(map.get("sum"), Some(TreeNode::Leaf(_))));

                // cumulative: Height merged with rest's cumulative
                let cumulative_indexes = get_leaf_indexes(map.get("cumulative").unwrap()).unwrap();
                assert!(cumulative_indexes.contains(&Index::Height));
                assert!(cumulative_indexes.contains(&Index::Day1));
            }
            _ => panic!("Expected branch"),
        }
    }

    // ========== Case 9: ValueBlockSumCum (outer no merge, inner has merge) ==========
    // ValueBlockSumCum (no merge):
    //   - sats: ComputedBlockSumCum<Sats> (merge) → { base, sum, cumulative }
    //   - bitcoin: LazyBlockSumCum<Bitcoin> (merge) → { base, sum, cumulative }
    //   - dollars: Option<ComputedBlockSumCum<Dollars>> (merge) → { base, sum, cumulative }

    #[test]
    fn case9_value_block_sum_cum() {
        // Each denomination has already been merged internally
        // Simulating the output after inner merge
        let sats_merged = branch(vec![
            ("raw", leaf("s", Index::Height)),
            ("sum", leaf("s_sum", Index::Day1)),
            ("cumulative", leaf("s_cumulative", Index::Height)),
        ]);

        let bitcoin_merged = branch(vec![
            ("raw", leaf("s_btc", Index::Height)),
            ("sum", leaf("s_btc_sum", Index::Day1)),
            ("cumulative", leaf("s_btc_cumulative", Index::Height)),
        ]);

        let dollars_merged = branch(vec![
            ("raw", leaf("s_usd", Index::Height)),
            ("sum", leaf("s_usd_sum", Index::Day1)),
            ("cumulative", leaf("s_usd_cumulative", Index::Height)),
        ]);

        // Outer struct has no merge, so denominations stay as branches
        let tree = branch(vec![
            ("sats", sats_merged),
            ("bitcoin", bitcoin_merged),
            ("dollars", dollars_merged),
        ]);

        match &tree {
            TreeNode::Branch(map) => {
                assert_eq!(map.len(), 3);

                for denom in ["sats", "bitcoin", "dollars"] {
                    match map.get(denom) {
                        Some(TreeNode::Branch(inner)) => {
                            assert_eq!(inner.len(), 3);
                            assert!(inner.contains_key("raw"));
                            assert!(inner.contains_key("sum"));
                            assert!(inner.contains_key("cumulative"));
                        }
                        _ => panic!("Expected branch for {}", denom),
                    }
                }
            }
            _ => panic!("Expected branch"),
        }
    }

    // ========== Case 10: ValueDateLast structure ==========
    // Testing individual components of ValueDateLast

    #[test]
    fn case10_derived_date_last_collapses_to_leaf() {
        // DerivedDateLast<T> with merge: all fields have wrap="last"
        // week1: { last: Leaf }, month1: { last: Leaf }, etc.
        // After merge: all "last" keys merge, same series name → collapses to Leaf
        let tree = branch(vec![
            ("week1", branch(vec![("last", leaf("s", Index::Week1))])),
            ("month1", branch(vec![("last", leaf("s", Index::Month1))])),
            ("year1", branch(vec![("last", leaf("s", Index::Year1))])),
        ]);

        let merged = tree.merge_branches().unwrap();

        // Should collapse to single Leaf with all indexes
        match &merged {
            TreeNode::Leaf(leaf) => {
                let indexes = leaf.indexes();
                assert!(indexes.contains(&Index::Week1));
                assert!(indexes.contains(&Index::Month1));
                assert!(indexes.contains(&Index::Year1));
            }
            TreeNode::Branch(map) => {
                panic!(
                    "Expected Leaf, got Branch: {:?}",
                    map.keys().collect::<Vec<_>>()
                );
            }
        }
    }

    #[test]
    fn case10_computed_date_last_collapses_to_leaf() {
        // ComputedDateLast<T> with merge:
        //   - day1 with wrap="raw" → { base: Leaf }
        //   - rest (flatten): DerivedDateLast already merged to Leaf
        //     → flatten inserts with field name "rest" as key
        //
        // Both have same series name → collapses to single Leaf
        let tree = branch(vec![
            // day1 with wrap="raw"
            ("day1", branch(vec![("raw", leaf("s", Index::Day1))])),
            // rest (flatten): DerivedDateLast merged to Leaf
            // Same series name as base
            ("rest", leaf("s", Index::Week1)),
        ]);

        let merged = tree.merge_branches().unwrap();

        // Same series name → collapses to single Leaf with all indexes
        match &merged {
            TreeNode::Leaf(leaf) => {
                let indexes = leaf.indexes();
                assert!(indexes.contains(&Index::Day1));
                assert!(indexes.contains(&Index::Week1));
            }
            TreeNode::Branch(map) => {
                panic!(
                    "Expected Leaf, got Branch: {:?}",
                    map.keys().collect::<Vec<_>>()
                );
            }
        }
    }

    // ========== Case 11: ValueDateLast conflict detection ==========

    #[test]
    fn case11_value_date_last_sats_key_conflict() {
        // ValueDateLast has a structural issue:
        // - sats_day1 with wrap="sats" produces key "sats"
        // - rest (flatten) has field "sats" (DerivedDateLast<Sats>)
        // Both try to use the same "sats" key!

        // Simulating the pre-merge structure
        let tree = branch(vec![
            // From sats_day1 with wrap="sats"
            ("sats_day1", branch(vec![("sats", leaf("s", Index::Day1))])),
            // From rest (flatten): ValueDerivedDateLast
            (
                "rest",
                branch(vec![
                    // sats field: DerivedDateLast merged to Leaf
                    ("sats", leaf("s", Index::Week1)), // Same series name!
                    ("bitcoin", leaf("s_btc", Index::Day1)),
                    ("dollars", leaf("s_usd", Index::Day1)),
                ]),
            ),
        ]);

        let merged = tree.merge_branches();

        // Should succeed because both "sats" have the same series name
        // Indexes should be merged
        match merged {
            Some(TreeNode::Branch(map)) => {
                let sats_indexes = get_leaf_indexes(map.get("sats").unwrap()).unwrap();
                assert!(sats_indexes.contains(&Index::Day1));
                assert!(sats_indexes.contains(&Index::Week1));
            }
            Some(_) => panic!("Expected branch"),
            None => panic!("Unexpected conflict"),
        }
    }

    // ========== Case 12: ValueDateLast ideal output ==========

    #[test]
    fn case12_value_date_last_ideal_output() {
        // The IDEAL output for ValueDateLast:
        // { sats: Leaf(all indexes), bitcoin: Leaf(all indexes), dollars: Leaf(all indexes) }
        //
        // This requires:
        // 1. Each denomination collapses its time indexes into one Leaf
        // 2. Denominations stay as separate siblings

        // Simulating final merged output
        let tree = branch(vec![
            ("sats", leaf("s", Index::Day1)), // placeholder, would have all indexes
            ("bitcoin", leaf("s_btc", Index::Day1)),
            ("dollars", leaf("s_usd", Index::Day1)),
        ]);

        match &tree {
            TreeNode::Branch(map) => {
                assert_eq!(map.len(), 3);
                assert!(matches!(map.get("sats"), Some(TreeNode::Leaf(_))));
                assert!(matches!(map.get("bitcoin"), Some(TreeNode::Leaf(_))));
                assert!(matches!(map.get("dollars"), Some(TreeNode::Leaf(_))));
            }
            _ => panic!("Expected branch with 3 denomination leaves"),
        }
    }
}
