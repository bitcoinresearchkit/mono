//! Python tree structure generation.

use std::collections::BTreeSet;
use std::fmt::Write;

use bitview_types::TreeNode;

use crate::{
    ClientMetadata, LanguageSyntax, PatternField, PythonSyntax, build_child_path,
    generate_leaf_field, generate_tree_node_field, prepare_tree_node,
};

/// Generate tree classes
pub fn generate_tree_classes(output: &mut String, catalog: &TreeNode, metadata: &ClientMetadata) {
    writeln!(output, "# Series tree classes\n").unwrap();

    let pattern_lookup = metadata.pattern_lookup();
    let mut generated = BTreeSet::new();
    generate_tree_class(
        output,
        "SeriesTree",
        "",
        catalog,
        pattern_lookup,
        metadata,
        &mut generated,
    );
}

/// Recursively generate tree classes
fn generate_tree_class(
    output: &mut String,
    name: &str,
    path: &str,
    node: &TreeNode,
    pattern_lookup: &std::collections::BTreeMap<Vec<PatternField>, String>,
    metadata: &ClientMetadata,
    generated: &mut BTreeSet<String>,
) {
    let Some(ctx) = prepare_tree_node(node, name, path, pattern_lookup, metadata, generated) else {
        return;
    };

    // Generate child classes FIRST (post-order traversal)
    // This ensures children are defined before parent references them
    for child in &ctx.children {
        if child.should_inline {
            let child_path = build_child_path(path, child.name);
            generate_tree_class(
                output,
                &child.inline_type_name,
                &child_path,
                child.node,
                pattern_lookup,
                metadata,
                generated,
            );
        }
    }

    // THEN generate the current class (after all children are defined)
    writeln!(output, "class {}:", name).unwrap();
    writeln!(output, "    \"\"\"Series tree node.\"\"\"").unwrap();
    writeln!(output).unwrap();
    writeln!(
        output,
        "    def __init__(self, client: BitviewClient, base_path: str = ''):"
    )
    .unwrap();

    if ctx.children.is_empty() {
        writeln!(output, "        pass").unwrap();
    }

    let syntax = PythonSyntax;
    for child in &ctx.children {
        if child.is_leaf {
            if let TreeNode::Leaf(leaf) = child.node {
                generate_leaf_field(
                    output, &syntax, "client", child.name, leaf, metadata, "        ",
                );
            }
        } else if child.should_inline {
            let field_name = syntax.field_name(child.name);
            writeln!(
                output,
                "        self.{}: {} = {}(client)",
                field_name, child.inline_type_name, child.inline_type_name
            )
            .unwrap();
        } else {
            generate_tree_node_field(
                output,
                &syntax,
                &child.field,
                metadata,
                "        ",
                "client",
                &child.base_result,
            );
        }
    }

    writeln!(output).unwrap();
}
