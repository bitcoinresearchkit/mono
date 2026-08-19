//! JavaScript language syntax implementation.

use crate::{GenericSyntax, LanguageSyntax, to_camel_case};

/// JavaScript-specific code generation syntax.
pub struct JavaScriptSyntax;

impl LanguageSyntax for JavaScriptSyntax {
    fn field_name(&self, name: &str) -> String {
        to_camel_case(name)
    }

    fn path_expr(&self, base_var: &str, suffix: &str) -> String {
        // Convert base_var to camelCase for JavaScript
        let var_name = to_camel_case(base_var);
        format!("`${{{}}}{}`", var_name, suffix)
    }

    fn suffix_expr(&self, acc_var: &str, relative: &str) -> String {
        let var_name = to_camel_case(acc_var);
        if relative.is_empty() {
            // Identity: just return acc
            var_name
        } else {
            // _m(acc, relative) -> acc ? `${acc}_relative` : 'relative'
            format!("_m({}, '{}')", var_name, relative)
        }
    }

    fn prefix_expr(&self, prefix: &str, acc_var: &str) -> String {
        let var_name = to_camel_case(acc_var);
        if prefix.is_empty() {
            // Identity: just return acc
            var_name
        } else {
            // _p(prefix, acc) -> acc ? `${prefix}${acc}` : 'prefix_without_underscore'
            let prefix_base = prefix.trim_end_matches('_');
            format!("_p('{}', {})", prefix_base, var_name)
        }
    }

    fn constructor(&self, type_name: &str, path_expr: &str) -> String {
        format!("create{}(client, {})", type_name, path_expr)
    }

    fn field_init(&self, indent: &str, name: &str, _type_ann: &str, value: &str) -> String {
        // JavaScript uses object literal syntax; type is in JSDoc, not in assignment
        format!("{}{}: {},", indent, name, value)
    }

    fn generic_syntax(&self) -> GenericSyntax {
        GenericSyntax::JAVASCRIPT
    }

    fn string_literal(&self, value: &str) -> String {
        format!("'{}'", value)
    }

    fn constructor_name(&self, type_name: &str) -> String {
        format!("create{}", type_name)
    }

    fn disc_arg_expr(&self, template: &str) -> String {
        if template == "{disc}" {
            "disc".to_string()
        } else if template.is_empty() {
            "''".to_string()
        } else if !template.contains("{disc}") {
            format!("'{}'", template)
        } else if template.ends_with("{disc}") {
            let static_part = template.trim_end_matches("{disc}").trim_end_matches('_');
            format!("_m('{}', disc)", static_part)
        } else {
            let js_template = template.replace("{disc}", "${disc}");
            format!("`{}`", js_template)
        }
    }

    fn template_expr(&self, acc_var: &str, template: &str) -> String {
        let var_name = to_camel_case(acc_var);
        if template.is_empty() {
            // Identity — just pass disc
            format!("_m({}, disc)", var_name)
        } else if template == "{disc}" {
            // Template IS the discriminator
            format!("_m({}, disc)", var_name)
        } else if !template.contains("{disc}") {
            // Static suffix — no disc involved
            format!("_m({}, '{}')", var_name, template)
        } else {
            // Template with {disc}: use nested _m for proper separator handling
            // "ratio_{disc}_ppm" → split on {disc} → _m(_m(acc, 'ratio'), disc) then _ppm
            // But this is complex. For embedded disc, use string interpolation.
            // For suffix disc (ends with {disc}), use _m composition.
            if let Some(static_part) = template.strip_suffix("{disc}") {
                if static_part.is_empty() {
                    format!("_m({}, disc)", var_name)
                } else {
                    let static_part = static_part.trim_end_matches('_');
                    format!("_m(_m({}, '{}'), disc)", var_name, static_part)
                }
            } else {
                // Embedded disc — use template literal
                let js_template = template.replace("{disc}", "${disc}");
                format!("_m({}, `{}`)", var_name, js_template)
            }
        }
    }
}
