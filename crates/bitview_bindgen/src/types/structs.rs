//! Structural pattern and field types.

use std::collections::{BTreeMap, BTreeSet};

use brk_types::Index;

use super::PatternMode;

/// A pattern of indexes that appear together on multiple series.
#[derive(Debug, Clone)]
pub struct IndexSetPattern {
    /// Pattern name (e.g., "DateHeightIndexes")
    pub name: String,
    /// The set of indexes
    pub indexes: BTreeSet<Index>,
}

/// A structural pattern - a branch structure that appears multiple times.
#[derive(Debug, Clone)]
pub struct StructuralPattern {
    /// Pattern name
    pub name: String,
    /// Ordered list of child fields
    pub fields: Vec<PatternField>,
    /// How fields construct series names from acc (None = not parameterizable)
    pub mode: Option<PatternMode>,
    /// If true, all leaf fields use a type parameter T
    pub is_generic: bool,
}

impl StructuralPattern {
    /// Returns true if this pattern can be parameterized with an accumulator.
    pub fn is_parameterizable(&self) -> bool {
        self.mode.is_some()
    }

    /// Get the field part (relative name or prefix) for a given field.
    pub fn get_field_part(&self, field_name: &str) -> Option<&str> {
        match &self.mode {
            Some(PatternMode::Suffix { relatives }) => {
                relatives.get(field_name).map(|s| s.as_str())
            }
            Some(PatternMode::Prefix { prefixes }) => prefixes.get(field_name).map(|s| s.as_str()),
            Some(PatternMode::Templated { templates }) => {
                templates.get(field_name).map(|s| s.as_str())
            }
            None => None,
        }
    }

    /// Returns true if this pattern is in suffix mode.
    pub fn is_suffix_mode(&self) -> bool {
        matches!(
            &self.mode,
            Some(PatternMode::Suffix { .. } | PatternMode::Templated { .. })
        )
    }

    /// Returns true if this pattern uses templated mode with a discriminator.
    pub fn is_templated(&self) -> bool {
        matches!(&self.mode, Some(PatternMode::Templated { .. }))
    }

    /// Extract the discriminator value from a concrete instance's field_parts.
    /// Uses the pattern's templates to reverse-match and find the disc.
    pub fn extract_disc_from_instance(
        &self,
        instance_field_parts: &BTreeMap<String, String>,
    ) -> Option<String> {
        let templates = match &self.mode {
            Some(PatternMode::Templated { templates }) => templates,
            _ => return None,
        };
        // Find a template with {disc} and extract the disc from the instance value.
        // Strip leading underscore since _m() handles separators.
        for (field_name, template) in templates {
            if let Some(value) = instance_field_parts.get(field_name)
                && let Some(disc) = extract_disc(template, value)
            {
                return Some(disc.trim_start_matches('_').to_string());
            }
        }
        // If no template matched (all empty templates), disc is empty
        Some(String::new())
    }

    /// Check if the given instance field parts match this pattern's field parts.
    pub fn field_parts_match(&self, instance_field_parts: &BTreeMap<String, String>) -> bool {
        match &self.mode {
            Some(PatternMode::Suffix { relatives }) => {
                relatives.iter().all(|(field_name, pattern_suffix)| {
                    instance_field_parts
                        .get(field_name)
                        .is_some_and(|instance_suffix| instance_suffix == pattern_suffix)
                })
            }
            Some(PatternMode::Prefix { prefixes }) => {
                prefixes.iter().all(|(field_name, pattern_prefix)| {
                    instance_field_parts
                        .get(field_name)
                        .is_some_and(|instance_prefix| instance_prefix == pattern_prefix)
                })
            }
            Some(PatternMode::Templated { templates }) => {
                // For templated patterns, check if the instance's field_parts
                // can be produced by substituting some discriminator into the templates
                let first_template_field = templates.iter().next();
                let Some((ref_field, ref_template)) = first_template_field else {
                    return false;
                };
                let Some(ref_value) = instance_field_parts.get(ref_field) else {
                    return false;
                };
                // Extract discriminator from the reference field
                let Some(disc) = extract_disc(ref_template, ref_value) else {
                    return false;
                };
                // Verify all fields match with this discriminator
                templates.iter().all(|(field_name, template)| {
                    instance_field_parts
                        .get(field_name)
                        .is_some_and(|value| *value == template.replace("{disc}", &disc))
                })
            }
            None => false,
        }
    }
}

/// Extract the discriminator value by matching a template against a concrete string.
/// E.g., template `"ratio_{disc}_ppm"` matched against `"ratio_pct99_ppm"` yields `"pct99"`.
fn extract_disc(template: &str, value: &str) -> Option<String> {
    let (prefix, suffix) = template.split_once("{disc}")?;
    if value.starts_with(prefix) && value.ends_with(suffix) {
        let disc = &value[prefix.len()..value.len() - suffix.len()];
        if !disc.is_empty() {
            return Some(disc.to_string());
        }
    }
    None
}

/// A field in a structural pattern.
#[derive(Debug, Clone, PartialOrd, Ord)]
pub struct PatternField {
    /// Field name
    pub name: String,
    /// Rust type for leaves or pattern name for branches
    pub rust_type: String,
    /// JSON type from schema
    pub json_type: String,
    /// For leaves: the set of supported indexes. Empty for branches.
    pub indexes: BTreeSet<Index>,
    /// For branches referencing generic patterns: the concrete type parameter
    pub type_param: Option<String>,
}

impl PatternField {
    /// Returns true if this is a leaf field (has indexes).
    pub fn is_leaf(&self) -> bool {
        !self.indexes.is_empty()
    }

    /// Returns true if this is a branch field (no indexes).
    pub fn is_branch(&self) -> bool {
        self.indexes.is_empty()
    }
}

impl std::hash::Hash for PatternField {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.rust_type.hash(state);
        self.json_type.hash(state);
        self.indexes.hash(state);
    }
}

impl PartialEq for PatternField {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
            && self.rust_type == other.rust_type
            && self.json_type == other.json_type
            && self.indexes == other.indexes
    }
}

impl Eq for PatternField {}
