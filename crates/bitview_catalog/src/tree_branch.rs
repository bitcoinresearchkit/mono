use std::ops::{Deref, DerefMut};

use indexmap::{IndexMap, map};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::TreeNode;

// Catalog children and their internal naming metadata.
// Keep the schema transparent too: schema documentation belongs to TreeNode,
// not this wrapper, so its schema identity stays identical to the original map.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema)]
#[serde(transparent)]
pub struct TreeBranch {
    pub children: IndexMap<String, TreeNode>,
    /// Each child's base is the parent base followed by `_` and its catalog key.
    #[serde(skip)]
    pub field_suffixes: bool,
}

// As with leaf schemas, internal metadata does not change catalog equality.
impl PartialEq for TreeBranch {
    fn eq(&self, other: &Self) -> bool {
        self.children == other.children
    }
}

impl Eq for TreeBranch {}

impl From<IndexMap<String, TreeNode>> for TreeBranch {
    fn from(children: IndexMap<String, TreeNode>) -> Self {
        Self {
            children,
            field_suffixes: false,
        }
    }
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

impl Deref for TreeBranch {
    type Target = IndexMap<String, TreeNode>;

    fn deref(&self) -> &Self::Target {
        &self.children
    }
}

impl DerefMut for TreeBranch {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.children
    }
}

impl IntoIterator for TreeBranch {
    type Item = (String, TreeNode);
    type IntoIter = map::IntoIter<String, TreeNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.into_iter()
    }
}

impl<'a> IntoIterator for &'a TreeBranch {
    type Item = (&'a String, &'a TreeNode);
    type IntoIter = map::Iter<'a, String, TreeNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.iter()
    }
}

impl<'a> IntoIterator for &'a mut TreeBranch {
    type Item = (&'a String, &'a mut TreeNode);
    type IntoIter = map::IterMut<'a, String, TreeNode>;

    fn into_iter(self) -> Self::IntoIter {
        self.children.iter_mut()
    }
}
