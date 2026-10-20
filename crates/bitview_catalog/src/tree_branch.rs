use derive_more::{Deref, DerefMut, IntoIterator};
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::TreeNode;

// Catalog children, merged by key, and the Rust type that defined them. A plain comment: a doc
// comment would add schema metadata, and this wrapper's schema stays identical to the bare map's.
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    Deref,
    DerefMut,
    IntoIterator,
)]
#[serde(transparent)]
pub struct TreeBranch {
    #[deref]
    #[deref_mut]
    #[into_iterator(owned, ref, ref_mut)]
    pub children: IndexMap<String, TreeNode>,
    // The name client types take for the branch (its defining type's, `PoolVecs` as `Pool`, or one
    // it declares); none for a map or a wrapper. Not part of the catalog's JSON.
    #[serde(skip)]
    #[schemars(skip)]
    pub client_name: Option<&'static str>,
}

impl From<IndexMap<String, TreeNode>> for TreeBranch {
    fn from(children: IndexMap<String, TreeNode>) -> Self {
        Self {
            children,
            client_name: None,
        }
    }
}

impl TreeBranch {
    /// Panics on a conflicting key (see [`TreeNode::merge_branches`]).
    pub fn merge_field(&mut self, key: String, node: TreeNode) {
        TreeNode::merge_node(&mut self.children, key, node);
    }

    pub fn merge_fields(&mut self, other: Self) {
        for (key, child) in other.children {
            self.merge_field(key, child);
        }
    }
}

impl FromIterator<(String, TreeNode)> for TreeBranch {
    fn from_iter<T: IntoIterator<Item = (String, TreeNode)>>(iter: T) -> Self {
        Self::from(iter.into_iter().collect::<IndexMap<_, _>>())
    }
}
