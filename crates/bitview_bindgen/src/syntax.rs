//! Language-specific syntax traits for code generation.
//!
//! This module defines the `LanguageSyntax` trait that abstracts over
//! language-specific code generation patterns, allowing shared generation
//! logic to work across Python, JavaScript, and Rust backends.

use crate::GenericSyntax;

/// Language-specific syntax for code generation.
///
/// Implementations of this trait provide the language-specific formatting
/// for generated client code. This allows the core generation logic to be
/// written once and reused across all supported languages.
pub trait LanguageSyntax {
    /// Convert a field name to the language's naming convention.
    ///
    /// - Python/Rust: `snake_case`
    /// - JavaScript: `camelCase`
    fn field_name(&self, name: &str) -> String;

    /// Format an interpolated path expression.
    ///
    /// # Arguments
    /// * `base_var` - The variable name to interpolate (e.g., "acc", "base_path")
    /// * `suffix` - The suffix to append (e.g., "_field_name")
    ///
    /// # Returns
    /// - Python: `f'{acc}_suffix'`
    /// - JavaScript: `` `${acc}_suffix` ``
    /// - Rust: `format!("{acc}_suffix")`
    fn path_expr(&self, base_var: &str, suffix: &str) -> String;

    /// Format a suffix mode expression: `_m(acc, relative)`.
    ///
    /// Suffix mode appends the relative name to the accumulator.
    /// - If relative is empty, returns just acc (identity)
    /// - Otherwise: `{acc}_{relative}` or `{relative}` if acc is empty
    ///
    /// # Arguments
    /// * `acc_var` - The accumulator variable name (e.g., "acc")
    /// * `relative` - The relative name to append (e.g., "max_cost_basis")
    fn suffix_expr(&self, acc_var: &str, relative: &str) -> String;

    /// Format a prefix mode expression: `_p(prefix, acc)`.
    ///
    /// Prefix mode prepends the prefix to the accumulator.
    /// - If prefix is empty, returns just acc (identity)
    /// - Otherwise: `{prefix}{acc}` (prefix includes trailing underscore)
    ///
    /// # Arguments
    /// * `prefix` - The prefix to prepend (e.g., "cumulative_")
    /// * `acc_var` - The accumulator variable name (e.g., "acc")
    fn prefix_expr(&self, prefix: &str, acc_var: &str) -> String;

    /// Generate a constructor call for patterns and accessors.
    ///
    /// - Python: `TypeName(client, path)`
    /// - JavaScript: `createTypeName(client, path)`
    /// - Rust: `TypeName::new(client.clone(), path)`
    fn constructor(&self, type_name: &str, path_expr: &str) -> String;

    /// Generate a field initialization line.
    ///
    /// # Arguments
    /// * `indent` - The indentation string
    /// * `name` - The field name (already converted to language convention)
    /// * `type_ann` - The type annotation (may be ignored by some languages)
    /// * `value` - The initialization value/expression
    ///
    /// # Returns
    /// - Python: `{indent}self.{name}: {type_ann} = {value}`
    /// - JavaScript: `{indent}{name}: {value},`
    /// - Rust: `{indent}{name}: {value},`
    fn field_init(&self, indent: &str, name: &str, type_ann: &str, value: &str) -> String;

    /// Get the generic type syntax for this language.
    ///
    /// - Python: `[T]` with default `Any`
    /// - JavaScript: `<T>` with default `unknown`
    /// - Rust: `<T>` with default `_`
    fn generic_syntax(&self) -> GenericSyntax;

    /// Format a string literal.
    ///
    /// - Python/JavaScript: `'value'` (single quotes)
    /// - Rust: `"value"` (double quotes)
    fn string_literal(&self, value: &str) -> String;

    /// Get the constructor name/prefix for a type.
    ///
    /// - Python: `TypeName`
    /// - JavaScript: `createTypeName`
    /// - Rust: `TypeName::new`
    fn constructor_name(&self, type_name: &str) -> String;

    /// Return a variable as an owned value expression.
    ///
    /// - Rust: `var.clone()` (String needs explicit cloning)
    /// - JavaScript/Python: `var` (no ownership)
    fn owned_expr(&self, var: &str) -> String {
        var.to_string()
    }

    /// Format a discriminator argument for passing to a templated child.
    ///
    /// Returns an expression computing the disc value from a template.
    /// - `"pct99"` (static) → `'pct99'` (JS) / `"pct99".to_string()` (Rust)
    /// - `""` (empty) → `disc` (pass parent's disc through)
    /// - `"p1sd{disc}"` (suffix) → `_m('p1sd', disc)` (composed)
    /// - `"ratio_{disc}_ppm"` (embedded) → `` `ratio_${disc}_ppm` `` (template literal)
    fn disc_arg_expr(&self, template: &str) -> String;

    /// Format a templated mode expression: substitute `{disc}` at runtime.
    ///
    /// The template contains `{disc}` placeholder. The generated code should
    /// construct `_m(acc, template_with_disc_substituted)` at runtime.
    ///
    /// # Arguments
    /// * `acc_var` - The accumulator variable (e.g., "acc")
    /// * `template` - Template like `"ratio_{disc}_ppm"` or `"{disc}"`
    fn template_expr(&self, acc_var: &str, template: &str) -> String;
}

#[cfg(test)]
mod tests {
    use super::LanguageSyntax;
    use crate::{JavaScriptSyntax, PythonSyntax, RustSyntax};

    #[test]
    fn trailing_discriminator_is_a_separate_name_part() {
        let template = "2009_transfer_volume{disc}";

        assert_eq!(
            RustSyntax.template_expr("acc", template),
            "_m(&_m(&acc, \"2009_transfer_volume\"), &disc)"
        );
        assert_eq!(
            PythonSyntax.template_expr("acc", template),
            "_m(_m(acc, '2009_transfer_volume'), disc)"
        );
        assert_eq!(
            JavaScriptSyntax.template_expr("acc", template),
            "_m(_m(acc, '2009_transfer_volume'), disc)"
        );
    }
}
