use std::collections::BTreeMap;

use bitview_catalog::TreeNode;

use super::{CatalogFamily, CatalogNode, CatalogType, CatalogValue};
use crate::{IndexSetPattern, to_pascal_case};

/// Compiled, lossless catalog bindings and their structural record families.
#[derive(Debug, Clone, Default)]
pub struct CatalogTree {
    pub families: Vec<CatalogFamily>,
    pub types: Vec<CatalogType>,
    pub nodes: Vec<CatalogNode>,
    pub root: usize,
}

#[derive(Default)]
struct Compiler {
    tree: CatalogTree,
    /// A family is its ordered fields and their type-parameter slots, nothing else.
    families: BTreeMap<Vec<(String, usize)>, usize>,
    types: BTreeMap<CatalogType, usize>,
    nodes: BTreeMap<CatalogNode, usize>,
}

impl CatalogTree {
    pub fn from_catalog(catalog: &TreeNode, indexes: &[IndexSetPattern]) -> Self {
        let mut compiler = Compiler::default();
        compiler.tree.root = compiler.compile(catalog, &mut Vec::new(), indexes);
        compiler.name_families();
        compiler.tree
    }
}

impl Compiler {
    fn compile(
        &mut self,
        node: &TreeNode,
        path: &mut Vec<String>,
        indexes: &[IndexSetPattern],
    ) -> usize {
        let (ty, value) = match node {
            TreeNode::Leaf(leaf) => (
                CatalogType::Leaf {
                    accessor: indexes
                        .iter()
                        .position(|pattern| &pattern.indexes == leaf.indexes())
                        .expect("Every catalog index set must have an accessor"),
                    value: leaf.kind().to_string(),
                },
                CatalogValue::Leaf(leaf.name().to_string()),
            ),
            TreeNode::Branch(branch) => {
                let mut fields = Vec::new();
                let mut children = Vec::new();
                let mut arguments = Vec::new();
                let mut slots = BTreeMap::new();
                for (key, child) in branch {
                    path.push(key.clone());
                    let child_id = self.compile(child, path, indexes);
                    path.pop();
                    let child_type = self.tree.nodes[child_id].type_id;
                    // Fields with the same projected type share one type parameter.
                    let parameter = *slots.entry(child_type).or_insert_with(|| {
                        arguments.push(child_type);
                        arguments.len() - 1
                    });
                    fields.push((key.clone(), parameter));
                    children.push(child_id);
                }
                let family = self.family(fields, path);
                (
                    CatalogType::Branch { family, arguments },
                    CatalogValue::Branch(children),
                )
            }
        };
        let type_id = *self.types.entry(ty.clone()).or_insert_with(|| {
            let id = self.tree.types.len();
            self.tree.types.push(ty);
            id
        });
        let node = CatalogNode { type_id, value };
        *self.nodes.entry(node.clone()).or_insert_with(|| {
            let id = self.tree.nodes.len();
            self.tree.nodes.push(node);
            id
        })
    }

    fn family(&mut self, fields: Vec<(String, usize)>, path: &[String]) -> usize {
        if let Some(&id) = self.families.get(&fields) {
            // Keep the canonical path: shallowest, then smallest.
            let canonical = &mut self.tree.families[id].path;
            if (path.len(), path) < (canonical.len(), canonical.as_slice()) {
                *canonical = path.to_vec();
            }
            return id;
        }
        let id = self.tree.families.len();
        self.tree.families.push(CatalogFamily {
            name: String::new(),
            path: path.to_vec(),
            fields: fields.clone(),
        });
        self.families.insert(fields, id);
        id
    }

    /// Names each family after the tail of its canonical path, using as many trailing
    /// segments as needed to be unique. Distinct paths whose names normalize the same
    /// (e.g. `a_b.c` and `a.b_c`) get a numeric suffix, ordered by path.
    fn name_families(&mut self) {
        let families = &mut self.tree.families;
        let mut depths = vec![1; families.len()];
        let owners = loop {
            let mut owners = BTreeMap::<String, Vec<usize>>::new();
            for (id, (family, &depth)) in families.iter().zip(&depths).enumerate() {
                owners
                    .entry(family_name(&family.path, depth))
                    .or_default()
                    .push(id);
            }
            let mut deepened = false;
            for ids in owners.values().filter(|ids| ids.len() > 1) {
                for &id in ids {
                    if depths[id] < families[id].path.len() {
                        depths[id] += 1;
                        deepened = true;
                    }
                }
            }
            if !deepened {
                break owners;
            }
        };

        for (name, mut ids) in owners {
            ids.sort_by(|&a, &b| families[a].path.cmp(&families[b].path));
            for (rank, id) in ids.into_iter().enumerate() {
                families[id].name = if rank == 0 {
                    name.clone()
                } else {
                    format!("{name}_{}", rank + 1)
                };
            }
        }
    }
}

fn family_name(path: &[String], depth: usize) -> String {
    let tail = &path[path.len() - depth.min(path.len())..];
    if tail.is_empty() {
        return "CatalogRoot".to_owned();
    }
    let words = tail
        .join("_")
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();
    format!("Catalog{}", to_pascal_case(&words))
}
