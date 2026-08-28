//! Shared field generation logic.
//!
//! This module contains the core field generation logic that is shared
//! across all language backends. The `LanguageSyntax` trait is used to
//! abstract over language-specific formatting.

use std::fmt::Write;

use bitview_types::SeriesLeafWithSchema;

use crate::{
    ClientMetadata, LanguageSyntax, PatternBaseResult, PatternField, PatternMode, StructuralPattern,
};

use super::FieldParts;

/// Create a path suffix from a name.
fn path_suffix(name: &str) -> String {
    if name.starts_with('_') {
        name.to_string()
    } else {
        format!("_{}", name)
    }
}

/// Compute the constructor value for a parameterized field (factory context).
///
/// Handles all three pattern modes (Suffix/Prefix/Templated) and the special
/// case of templated child patterns that need (acc, disc) instead of a path.
fn compute_parameterized_value<S: LanguageSyntax>(
    syntax: &S,
    field: &PatternField,
    pattern: &StructuralPattern,
    metadata: &ClientMetadata,
) -> String {
    // Templated child patterns receive acc and disc as separate arguments.
    // A regular suffix/prefix on the parent is already part of the child's
    // base path, while a templated parent is passing a true discriminator.
    if let Some(child_pattern) = metadata.find_pattern(&field.rust_type)
        && child_pattern.is_templated()
    {
        let part = pattern.get_field_part(&field.name).unwrap_or(&field.name);
        let (acc_arg, disc_arg) = templated_child_args(syntax, pattern, child_pattern, part);
        return syntax.constructor(&field.rust_type, &format!("{acc_arg}, {disc_arg}"));
    }

    // Compute path expression from pattern mode
    let path_expr = match pattern.get_field_part(&field.name) {
        Some(part) => match &pattern.mode {
            Some(PatternMode::Templated { .. }) => syntax.template_expr("acc", part),
            Some(PatternMode::Prefix { .. }) => syntax.prefix_expr(part, "acc"),
            _ => syntax.suffix_expr("acc", part),
        },
        None => syntax.path_expr("acc", &path_suffix(&field.name)),
    };

    // Wrap in constructor — leaves use their index accessor, everything else uses the type name
    if let Some(accessor) = metadata.find_index_set_pattern(&field.indexes) {
        syntax.constructor(&accessor.name, &path_expr)
    } else if field.is_leaf() {
        panic!(
            "Field '{}' has no matching index accessor. All series must be indexed.",
            field.name
        )
    } else {
        syntax.constructor(&field.rust_type, &path_expr)
    }
}

fn templated_child_args<S: LanguageSyntax>(
    syntax: &S,
    parent: &StructuralPattern,
    child: &StructuralPattern,
    part: &str,
) -> (String, String) {
    let child_needs_positioned_disc = match &child.mode {
        Some(PatternMode::Templated { templates }) => templates.values().any(|template| {
            template == "{disc}" || (template.contains("{disc}") && !template.ends_with("{disc}"))
        }),
        _ => false,
    };

    if child_needs_positioned_disc {
        return (syntax.owned_expr("acc"), syntax.disc_arg_expr(part));
    }

    match &parent.mode {
        Some(PatternMode::Suffix { .. }) => {
            (syntax.suffix_expr("acc", part), syntax.disc_arg_expr(""))
        }
        Some(PatternMode::Prefix { .. }) => {
            (syntax.prefix_expr(part, "acc"), syntax.disc_arg_expr(""))
        }
        Some(PatternMode::Templated { .. }) | None => {
            (syntax.owned_expr("acc"), syntax.disc_arg_expr(part))
        }
    }
}

/// Generate a parameterized field for a pattern factory.
///
/// Used for pattern instances where fields build series names from an accumulated base.
pub fn generate_parameterized_field<S: LanguageSyntax>(
    output: &mut String,
    syntax: &S,
    field: &PatternField,
    pattern: &StructuralPattern,
    metadata: &ClientMetadata,
    indent: &str,
) {
    let field_name = syntax.field_name(&field.name);
    let type_ann =
        metadata.field_type_annotation(field, pattern.is_generic, None, syntax.generic_syntax());
    let value = compute_parameterized_value(syntax, field, pattern, metadata);

    writeln!(
        output,
        "{}",
        syntax.field_init(indent, &field_name, &type_ann, &value)
    )
    .unwrap();
}

/// Build the language-specific parts of a pattern tree-node field.
pub fn tree_node_field_parts<S: LanguageSyntax>(
    syntax: &S,
    field: &PatternField,
    metadata: &ClientMetadata,
    client_expr: &str,
    base_result: &PatternBaseResult,
) -> FieldParts {
    let field_name = syntax.field_name(&field.name);
    let type_annotation =
        metadata.field_type_annotation(field, false, None, syntax.generic_syntax());
    let base_arg = syntax.string_literal(&base_result.base);

    let value = if let Some(pattern) = metadata.find_pattern(&field.rust_type)
        && pattern.is_templated()
    {
        let disc = pattern
            .extract_disc_from_instance(&base_result.field_parts)
            .unwrap_or_default();
        format!(
            "{}({}, {}, {})",
            syntax.constructor_name(&field.rust_type),
            client_expr,
            base_arg,
            syntax.string_literal(&disc)
        )
    } else {
        format!(
            "{}({}, {})",
            syntax.constructor_name(&field.rust_type),
            client_expr,
            base_arg
        )
    };

    FieldParts {
        name: field_name,
        type_annotation,
        value,
    }
}

/// Generate a tree node field for a pattern-type child.
///
/// Called for non-inline branch children that match a parameterizable pattern.
/// For templated patterns, extracts the discriminator from the base result.
pub fn generate_tree_node_field<S: LanguageSyntax>(
    output: &mut String,
    syntax: &S,
    field: &PatternField,
    metadata: &ClientMetadata,
    indent: &str,
    client_expr: &str,
    base_result: &PatternBaseResult,
) {
    let FieldParts {
        name,
        type_annotation,
        value,
    } = tree_node_field_parts(syntax, field, metadata, client_expr, base_result);

    writeln!(
        output,
        "{}",
        syntax.field_init(indent, &name, &type_annotation, &value)
    )
    .unwrap();
}

/// Build the language-specific parts of a leaf field.
pub fn leaf_field_parts<S: LanguageSyntax>(
    syntax: &S,
    client_expr: &str,
    tree_field_name: &str,
    leaf: &SeriesLeafWithSchema,
    metadata: &ClientMetadata,
) -> FieldParts {
    let field_name = syntax.field_name(tree_field_name);
    let accessor = metadata
        .find_index_set_pattern(leaf.indexes())
        .unwrap_or_else(|| {
            panic!(
                "Series '{}' has no matching index pattern. All series must be indexed.",
                leaf.name()
            )
        });

    let type_annotation = metadata.field_type_annotation_from_leaf(leaf, syntax.generic_syntax());
    let series_name = syntax.string_literal(leaf.name());
    let value = format!(
        "{}({}, {})",
        syntax.constructor_name(&accessor.name),
        client_expr,
        series_name
    );

    FieldParts {
        name: field_name,
        type_annotation,
        value,
    }
}

/// Generate a leaf field using the actual series name from the TreeNode::Leaf.
///
/// This is the shared implementation for all language backends. It uses
/// `leaf.name()` directly to get the correct series name, avoiding any
/// path concatenation that could produce incorrect names.
///
/// # Arguments
/// * `output` - The string buffer to write to
/// * `syntax` - The language syntax implementation
/// * `client_expr` - The client expression (e.g., "client.clone()", "this", "client")
/// * `tree_field_name` - The field name from the tree structure
/// * `leaf` - The Leaf node containing the actual series name and indexes
/// * `metadata` - Client metadata for looking up index patterns
/// * `indent` - Indentation string
pub fn generate_leaf_field<S: LanguageSyntax>(
    output: &mut String,
    syntax: &S,
    client_expr: &str,
    tree_field_name: &str,
    leaf: &SeriesLeafWithSchema,
    metadata: &ClientMetadata,
    indent: &str,
) {
    let FieldParts {
        name,
        type_annotation,
        value,
    } = leaf_field_parts(syntax, client_expr, tree_field_name, leaf, metadata);

    writeln!(
        output,
        "{}",
        syntax.field_init(indent, &name, &type_annotation, &value)
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use super::templated_child_args;
    use crate::{JavaScriptSyntax, PatternMode, StructuralPattern};

    fn pattern(name: &str, mode: PatternMode) -> StructuralPattern {
        StructuralPattern {
            name: name.to_string(),
            fields: Vec::new(),
            mode: Some(mode),
            is_generic: false,
        }
    }

    #[test]
    fn suffix_parent_moves_its_part_into_templated_child_base() {
        let parent = pattern(
            "Parent",
            PatternMode::Suffix {
                relatives: BTreeMap::new(),
            },
        );
        let child = pattern(
            "Child",
            PatternMode::Templated {
                templates: [(
                    "value".to_string(),
                    "10y_old_transfer_volume{disc}".to_string(),
                )]
                .into_iter()
                .collect(),
            },
        );

        assert_eq!(
            templated_child_args(&JavaScriptSyntax, &parent, &child, "over"),
            ("_m(acc, 'over')".to_string(), "''".to_string())
        );
    }

    #[test]
    fn templated_parent_forwards_its_part_as_child_discriminator() {
        let parent = pattern(
            "Parent",
            PatternMode::Templated {
                templates: BTreeMap::new(),
            },
        );
        let child = pattern(
            "Child",
            PatternMode::Templated {
                templates: [("value".to_string(), "value{disc}".to_string())]
                    .into_iter()
                    .collect(),
            },
        );

        assert_eq!(
            templated_child_args(&JavaScriptSyntax, &parent, &child, "pct99"),
            ("acc".to_string(), "'pct99'".to_string())
        );
    }

    #[test]
    fn suffix_parent_forwards_disc_when_child_uses_it_inside_field_parts() {
        let parent = pattern(
            "Parent",
            PatternMode::Suffix {
                relatives: BTreeMap::new(),
            },
        );
        let child = pattern(
            "Child",
            PatternMode::Templated {
                templates: [
                    ("price".to_string(), "{disc}".to_string()),
                    ("ppm".to_string(), "ratio_{disc}_ppm".to_string()),
                ]
                .into_iter()
                .collect(),
            },
        );

        assert_eq!(
            templated_child_args(&JavaScriptSyntax, &parent, &child, "pct99"),
            ("acc".to_string(), "'pct99'".to_string())
        );
    }
}
