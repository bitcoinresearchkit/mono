//! Client metadata extracted from the series catalog.

use std::collections::{BTreeMap, BTreeSet};

use bitview_catalog::{SeriesLeafWithSchema, TreeNode};
use brk_types::Index;

use super::{GenericSyntax, IndexSetPattern, PatternField, StructuralPattern, inner_type};
use crate::{PatternBaseResult, analysis};

/// Metadata extracted from the series catalog for client generation.
#[derive(Debug)]
pub struct ClientMetadata {
    /// The catalog tree structure (with schemas in leaves)
    pub(crate) catalog: TreeNode,
    /// Structural patterns - tree node shapes that repeat
    pub(crate) structural_patterns: Vec<StructuralPattern>,
    /// Index set patterns - sets of indexes that appear together on series
    pub(crate) index_set_patterns: Vec<IndexSetPattern>,
    /// Maps field signatures to pattern names (merged from concrete instances + pattern definitions)
    pattern_lookup: BTreeMap<Vec<PatternField>, String>,
    /// Maps concrete field signatures to their type parameter (for generic patterns)
    concrete_to_type_param: BTreeMap<Vec<PatternField>, String>,
    /// Maps tree paths to their computed PatternBaseResult
    node_bases: BTreeMap<String, PatternBaseResult>,
}

impl ClientMetadata {
    /// Extract metadata from a catalog TreeNode directly.
    pub(crate) fn from_catalog(catalog: TreeNode) -> Self {
        let (structural_patterns, concrete_to_pattern, concrete_to_type_param, node_bases) =
            analysis::detect_structural_patterns(&catalog);
        let index_set_patterns = analysis::detect_index_patterns(&catalog);

        // Build merged pattern lookup: concrete instances + pattern definitions
        let mut pattern_lookup = concrete_to_pattern;
        for p in &structural_patterns {
            pattern_lookup.insert(p.fields.clone(), p.name.clone());
        }

        ClientMetadata {
            catalog,
            structural_patterns,
            index_set_patterns,
            pattern_lookup,
            concrete_to_type_param,
            node_bases,
        }
    }

    /// Find an index set pattern that matches the given indexes.
    pub(crate) fn find_index_set_pattern(
        &self,
        indexes: &BTreeSet<Index>,
    ) -> Option<&IndexSetPattern> {
        self.index_set_patterns
            .iter()
            .find(|p| &p.indexes == indexes)
    }

    /// Find a pattern by name.
    pub(crate) fn find_pattern(&self, name: &str) -> Option<&StructuralPattern> {
        self.structural_patterns.iter().find(|p| p.name == name)
    }

    /// Check if a pattern is fully parameterizable (recursively).
    /// Returns false if the pattern or any nested branch pattern has no mode.
    pub(crate) fn is_parameterizable(&self, name: &str) -> bool {
        self.find_pattern(name)
            .is_some_and(|pattern| self.pattern_is_parameterizable(pattern))
    }

    fn pattern_is_parameterizable(&self, pattern: &StructuralPattern) -> bool {
        pattern.is_parameterizable()
            && pattern.fields.iter().all(|field| {
                !field.is_branch()
                    || self
                        .find_pattern(&field.rust_type)
                        .is_none_or(|child| self.pattern_is_parameterizable(child))
            })
    }

    /// Find a pattern by its concrete fields.
    pub(crate) fn find_pattern_by_fields(
        &self,
        fields: &[PatternField],
    ) -> Option<&StructuralPattern> {
        self.pattern_lookup
            .get(fields)
            .and_then(|name| self.find_pattern(name))
    }

    /// Get the type parameter for a generic pattern given its concrete fields.
    pub(crate) fn get_type_param(&self, fields: &[PatternField]) -> Option<&String> {
        self.concrete_to_type_param.get(fields)
    }

    /// Get the pre-computed pattern lookup map.
    pub(crate) fn pattern_lookup(&self) -> &BTreeMap<Vec<PatternField>, String> {
        &self.pattern_lookup
    }

    /// Get the pre-computed PatternBaseResult for a tree path.
    pub(crate) fn get_node_base(&self, path: &str) -> Option<&PatternBaseResult> {
        self.node_bases.get(path)
    }

    /// Generate type annotation for a field with language-specific syntax.
    pub(crate) fn field_type_annotation(
        &self,
        field: &PatternField,
        is_generic: bool,
        generic_value_type: Option<&str>,
        syntax: GenericSyntax,
    ) -> String {
        // Pattern type — single lookup instead of is_pattern_type + is_pattern_generic
        if let Some(pattern) = self.find_pattern(&field.rust_type) {
            if pattern.is_generic {
                let type_param = field
                    .type_param
                    .as_deref()
                    .or(generic_value_type)
                    .unwrap_or(if is_generic { "T" } else { syntax.default_type });
                return syntax.wrap(&field.rust_type, type_param);
            }
            return field.rust_type.clone();
        }

        // Branch type (non-pattern)
        if field.is_branch() {
            return field.rust_type.clone();
        }

        // Leaf type
        let value_type = if is_generic && field.rust_type == "T" {
            "T"
        } else {
            inner_type(&field.rust_type)
        };
        if let Some(accessor) = self.find_index_set_pattern(&field.indexes) {
            syntax.wrap(&accessor.name, value_type)
        } else {
            syntax.wrap("SeriesNode", value_type)
        }
    }

    /// Generate type annotation for a leaf node with language-specific syntax.
    ///
    /// This is a simpler version of `field_type_annotation` that works directly
    /// with a `SeriesLeafWithSchema` node instead of a `PatternField`.
    pub(crate) fn field_type_annotation_from_leaf(
        &self,
        leaf: &SeriesLeafWithSchema,
        syntax: GenericSyntax,
    ) -> String {
        let value_type = leaf.kind();
        if let Some(accessor) = self.find_index_set_pattern(leaf.indexes()) {
            syntax.wrap(&accessor.name, value_type)
        } else {
            syntax.wrap("SeriesNode", value_type)
        }
    }
}
