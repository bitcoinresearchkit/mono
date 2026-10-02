use std::collections::BTreeMap;

use super::{PatternField, PatternMode};

/// A structural pattern - a branch structure that appears multiple times.
#[derive(Debug, Clone)]
pub struct StructuralPattern {
    /// Pattern name
    pub(crate) name: String,
    /// Ordered list of child fields
    pub(crate) fields: Vec<PatternField>,
    /// How fields construct series names from acc (None = not parameterizable)
    pub(crate) mode: Option<PatternMode>,
    /// If true, all leaf fields use a type parameter T
    pub(crate) is_generic: bool,
}

impl StructuralPattern {
    /// Returns true if this pattern can be parameterized with an accumulator.
    pub(crate) fn is_parameterizable(&self) -> bool {
        self.mode.is_some()
    }

    /// Get the field part (relative name or prefix) for a given field.
    pub(crate) fn get_field_part(&self, field_name: &str) -> Option<&str> {
        let fields = match &self.mode {
            Some(PatternMode::Suffix { relatives }) => relatives,
            Some(PatternMode::Prefix { prefixes }) => prefixes,
            Some(PatternMode::Templated { templates }) => templates,
            None => return None,
        };
        fields.get(field_name).map(String::as_str)
    }

    /// Returns true if this pattern is in suffix mode.
    pub(crate) fn is_suffix_mode(&self) -> bool {
        matches!(
            &self.mode,
            Some(PatternMode::Suffix { .. } | PatternMode::Templated { .. })
        )
    }

    /// Returns true if this pattern uses templated mode with a discriminator.
    pub(crate) fn is_templated(&self) -> bool {
        matches!(&self.mode, Some(PatternMode::Templated { .. }))
    }

    /// Extract the discriminator value from a concrete instance's field_parts.
    /// Uses the pattern's templates to reverse-match and find the disc.
    pub(crate) fn extract_disc_from_instance(
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
    pub(crate) fn field_parts_match(
        &self,
        instance_field_parts: &BTreeMap<String, String>,
    ) -> bool {
        match &self.mode {
            Some(
                PatternMode::Suffix { relatives: parts } | PatternMode::Prefix { prefixes: parts },
            ) => parts
                .iter()
                .all(|(name, part)| instance_field_parts.get(name) == Some(part)),
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
    let disc = value.strip_prefix(prefix)?.strip_suffix(suffix)?;
    (!disc.is_empty()).then(|| disc.to_string())
}
