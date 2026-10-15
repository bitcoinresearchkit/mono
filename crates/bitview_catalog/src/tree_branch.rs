use derive_more::{Deref, DerefMut, From, IntoIterator};
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::TreeNode;

// Catalog children, merged by key.
// Keep the schema transparent too: schema documentation belongs to TreeNode,
// not this wrapper, so its schema identity stays identical to the original map.
#[derive(
    Debug,
    Clone,
    Default,
    PartialEq,
    Eq,
    Serialize,
    Deserialize,
    JsonSchema,
    From,
    Deref,
    DerefMut,
    IntoIterator,
)]
#[serde(transparent)]
#[into_iterator(owned, ref, ref_mut)]
pub struct TreeBranch {
    pub children: IndexMap<String, TreeNode>,
}

impl TreeBranch {
    /// Panics on a conflicting key (see [`TreeNode::merge_branches`]).
    pub fn merge_field(&mut self, key: String, node: TreeNode) {
        TreeNode::merge_node(&mut self.children, key, node);
    }

    /// Lift another branch's children into this one.
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
