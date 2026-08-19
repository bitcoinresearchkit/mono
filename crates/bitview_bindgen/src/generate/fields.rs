//! Shared field generation logic.
//!
//! This module contains the core field generation logic that is shared
//! across all language backends. The `LanguageSyntax` trait is used to
//! abstract over language-specific formatting.

use std::fmt::Write;

use brk_types::SeriesLeafWithSchema;

use crate::{
    ClientMetadata, LanguageSyntax, PatternBaseResult, PatternField, PatternMode, StructuralPattern,
};

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
    // Templated child patterns receive acc and disc as separate arguments
    if let Some(child_pattern) = metadata.find_pattern(&field.rust_type)
        && child_pattern.is_templated()
    {
        let disc_template = pattern.get_field_part(&field.name).unwrap_or(&field.name);
        let disc_arg = syntax.disc_arg_expr(disc_template);
        let acc_arg = syntax.owned_expr("acc");
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
    let field_name = syntax.field_name(&field.name);
    let type_ann = metadata.field_type_annotation(field, false, None, syntax.generic_syntax());
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

    writeln!(
        output,
        "{}",
        syntax.field_init(indent, &field_name, &type_ann, &value)
    )
    .unwrap();
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
    let field_name = syntax.field_name(tree_field_name);
    let accessor = metadata
        .find_index_set_pattern(leaf.indexes())
        .unwrap_or_else(|| {
            panic!(
                "Series '{}' has no matching index pattern. All series must be indexed.",
                leaf.name()
            )
        });

    let type_ann = metadata.field_type_annotation_from_leaf(leaf, syntax.generic_syntax());
    let series_name = syntax.string_literal(leaf.name());
    let value = format!(
        "{}({}, {})",
        syntax.constructor_name(&accessor.name),
        client_expr,
        series_name
    );

    writeln!(
        output,
        "{}",
        syntax.field_init(indent, &field_name, &type_ann, &value)
    )
    .unwrap();
}
