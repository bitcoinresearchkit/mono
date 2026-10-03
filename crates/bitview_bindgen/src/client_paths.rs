//! The typed series paths each generated client exposes.

use bitview_catalog::TreeNode;

use crate::{JavaScriptSyntax, LanguageSyntax, PythonSyntax, rust_field_name};

/// One series leaf as reached through the typed tree of each client, e.g.
/// `market.ath.days_since` (Rust, Python) and `market.ath.daysSince` (JavaScript).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClientPath {
    pub series: String,
    pub rust: String,
    pub javascript: String,
    pub python: String,
}

/// Every leaf's typed path, in catalog order, named exactly as the generators name fields.
pub fn client_paths(catalog: &TreeNode) -> Vec<ClientPath> {
    let mut paths = Vec::new();
    collect(catalog, &mut Vec::new(), &mut paths);
    paths
}

fn collect<'a>(node: &'a TreeNode, keys: &mut Vec<&'a str>, paths: &mut Vec<ClientPath>) {
    match node {
        TreeNode::Branch(branch) => {
            for (key, child) in branch.iter() {
                keys.push(key);
                collect(child, keys, paths);
                keys.pop();
            }
        }
        TreeNode::Leaf(leaf) => {
            let join = |name: &dyn Fn(&str) -> String| {
                keys.iter()
                    .map(|key| name(key))
                    .collect::<Vec<_>>()
                    .join(".")
            };
            paths.push(ClientPath {
                series: leaf.name().to_owned(),
                rust: join(&rust_field_name),
                javascript: join(&|key| JavaScriptSyntax.field_name(key)),
                python: join(&|key| PythonSyntax.field_name(key)),
            });
        }
    }
}
