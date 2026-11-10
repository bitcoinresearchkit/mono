use std::{
    collections::{BTreeMap, btree_map::Entry},
    ops::Deref,
};

use vecdb::{AnyExportableVec, ReadOnlyClone};

use crate::{Fragment, Traversable, TreeNode};

/// A public name beside the original data vector. An identity changes only the name.
#[derive(Clone, Copy)]
pub struct PublicSeries<'a> {
    pub name: &'a str,
    pub vec: &'a dyn AnyExportableVec,
    pub identity: bool,
}

impl<'a> PublicSeries<'a> {
    pub fn new(vec: &'a dyn AnyExportableVec) -> Self {
        Self {
            name: vec.name(),
            vec,
            identity: false,
        }
    }
}

/// One existing series, with all its indexes, published under another name. Its source
/// owns the values, versions and export names; no vector or transformation is introduced.
#[derive(Clone)]
pub struct NamedIdentity<T> {
    name: String,
    source: T,
}

impl<T: Traversable> NamedIdentity<T> {
    pub fn new(name: impl Into<String>, source: T) -> Self {
        let mut series = source.iter_public_series();
        if let Some(first) = series.next() {
            assert!(
                series.all(|series| series.name == first.name),
                "an identity names one series"
            );
        }
        drop(series);
        Self {
            name: name.into(),
            source,
        }
    }
}

impl<T> Deref for NamedIdentity<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.source
    }
}

impl<T: ReadOnlyClone> ReadOnlyClone for NamedIdentity<T> {
    type ReadOnly = NamedIdentity<T::ReadOnly>;

    fn read_only_clone(&self) -> Self::ReadOnly {
        NamedIdentity {
            name: self.name.clone(),
            source: self.source.read_only_clone(),
        }
    }
}

impl<T: Traversable> Traversable for NamedIdentity<T> {
    fn to_tree_node(&self) -> TreeNode {
        fn rename(node: &mut TreeNode, name: &str) {
            match node {
                TreeNode::Leaf(leaf) => leaf.leaf.name = name.to_owned(),
                TreeNode::Branch(branch) => {
                    for child in branch.children.values_mut() {
                        rename(child, name);
                    }
                }
            }
        }
        let mut node = self.source.to_tree_node();
        rename(&mut node, &self.name);
        node
    }

    fn iter_any_exportable(&self) -> impl Iterator<Item = &dyn AnyExportableVec> {
        self.source.iter_any_exportable()
    }

    fn iter_public_series(&self) -> impl Iterator<Item = PublicSeries<'_>> {
        self.source.iter_public_series().map(|series| PublicSeries {
            name: &self.name,
            vec: series.vec,
            identity: true,
        })
    }

    fn collect_series_descriptions<'a>(
        &'a self,
        fragments: &mut Vec<Fragment>,
        descriptions: &mut BTreeMap<&'a str, Vec<Fragment>>,
    ) {
        let mut source = BTreeMap::new();
        self.source
            .collect_series_descriptions(fragments, &mut source);
        for (_, fragments) in source {
            match descriptions.entry(&self.name) {
                Entry::Vacant(entry) => {
                    entry.insert(fragments);
                }
                Entry::Occupied(entry) => debug_assert_eq!(
                    entry.get(),
                    &fragments,
                    "conflicting identity descriptions for {}",
                    self.name
                ),
            }
        }
    }
}
